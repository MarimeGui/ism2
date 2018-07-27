extern crate clap;
extern crate ez_io;
extern crate half;
extern crate ism2;
extern crate my_gltf;

use clap::{App, Arg};
use ez_io::WriteE;
use half::f16;
use ism2::{
    joint_definition::JointAttribute, joint_definition::JointDefinitionSubSection,
    joint_definition::JointSubSection, model_data::MeshSubSection, model_data::SubSection,
    model_data::VerticesSubSection, ISM2, Section,
};
use my_gltf::{
    accessors::Accessor, asset::Asset, buffer_views::BufferView, buffers::Buffer, meshes::Mesh,
    meshes::Primitive, nodes::Node, scenes::Scene, GlTF,
};
use std::collections::HashMap;
use std::f32::consts::PI;
use std::fs::{create_dir_all, File};
use std::io::BufReader;
use std::path::Path;
use std::process::exit;

const DEG_TO_RAD: f32 = PI / 180f32;

#[derive(Clone)]
struct IVertex {
    position_coordinates: IPositionCoordinates,
    texture_coordinates: ITextureCoordinates,
}

#[derive(Clone)]
struct IPositionCoordinates {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Clone)]
struct ITextureCoordinates {
    u: f16,
    v: f16,
}

#[derive(Clone)]
struct IVerticesExtremes {
    x: IExtreme,
    y: IExtreme,
    z: IExtreme,
}

#[derive(Clone)]
struct IExtreme {
    min: f32,
    max: f32,
}

#[derive(Clone)]
struct IMesh {
    faces: Vec<(u16, u16, u16)>,
}

#[derive(Clone)]
struct INode {
    transform: Option<IPositionCoordinates>,
    rotation: Option<IQuaternion>,
    name: String,
    children: Vec<usize>,
}

#[derive(Clone)]
struct IQuaternion {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}

impl IVerticesExtremes {
    fn new(x: f32, y: f32, z: f32) -> IVerticesExtremes {
        IVerticesExtremes {
            x: IExtreme { min: x, max: x },
            y: IExtreme { min: y, max: y },
            z: IExtreme { min: z, max: z },
        }
    }
    fn to_vec(&self) -> Vec<Vec<f32>> {
        vec![
            vec![self.x.min, self.y.min, self.z.min],
            vec![self.x.max, self.y.max, self.z.max],
        ]
    }
}

impl IExtreme {
    fn update(&mut self, value: f32) {
        if self.min.gt(&value) {
            self.min = value
        }
        if self.max.lt(&value) {
            self.max = value
        }
    }
}

fn main() {
    let matches = App::new("ISM2 to GLTF Converter")
        .version("0.1")
        .author("Marime Gui")
        .about("It converts ISM2 files to GLTF files, should work with most files")
        .arg(
            Arg::with_name("INPUT")
                .help("ISM2 file to convert")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("OUTPUT")
                .help("Folder name for output")
                .required(true)
                .index(2),
        )
        .get_matches();

    // Create relevant paths
    let input_str = matches.value_of("INPUT").unwrap();
    let output_str = matches.value_of("OUTPUT").unwrap();
    let input_path = Path::new(input_str);
    let output_path = Path::new(output_str);
    if !input_path.exists() {
        eprintln!("Error: The specified input file does not exist or is unaccessible.");
        exit(1);
    }
    create_dir_all(output_path).unwrap();

    // Import ISM2 file
    let ism = ISM2::import(&mut BufReader::new(File::open(input_path).unwrap())).unwrap();

    // Create variables for defining what we want
    let mut i_vertices: Vec<IVertex> = Vec::new();
    let mut i_meshes: Vec<IMesh> = Vec::new();
    let mut i_nodes: Vec<INode> = Vec::new();
    let mut i_root_nodes: Vec<usize> = Vec::new();

    // Get the required information from the ISM file
    for section in ism.sections {
        match section {
            Section::ModelData(model_data) => {
                for sub_section in model_data.zero_a.sub_sections {
                    match sub_section {
                        SubSection::Vertices(vertices) => {
                            for vertices_sub_section in vertices.sub_sections {
                                match vertices_sub_section {
                                    VerticesSubSection::Data(data) => {
                                        for vertex in data.vertices {
                                            i_vertices.push(IVertex {
                                                position_coordinates: IPositionCoordinates {
                                                    x: vertex.position_coordinates[0],
                                                    y: vertex.position_coordinates[1],
                                                    z: vertex.position_coordinates[2],
                                                },
                                                texture_coordinates: ITextureCoordinates {
                                                    u: vertex.texture_coordinates[0],
                                                    v: vertex.texture_coordinates[1],
                                                },
                                            });
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        SubSection::Mesh(mesh) => {
                            let mut i_faces = Vec::new();
                            for sub_section in mesh.sub_sections {
                                match sub_section {
                                    MeshSubSection::Faces(faces) => {
                                        for face in faces.faces {
                                            i_faces.push(face.points);
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            i_meshes.push(IMesh { faces: i_faces });
                        }
                        _ => {}
                    }
                }
            }
            Section::JointDefinition(joint_definition) => {
                let mut i_children: Vec<Vec<usize>> = Vec::new();
                let mut id = 0usize;
                for sub_section in &joint_definition.sub_sections {
                    match sub_section {
                        JointDefinitionSubSection::Joint(joint) => {
                            i_children.push(Vec::new());
                            match joint.parent_index {
                                None => i_root_nodes.push(id),
                                Some(p) => match i_children.get_mut(p) {
                                    Some(ref mut c) => c.push(id),
                                    None => panic!("Missing parent"),
                                },
                            }
                            id += 1;
                        }
                        _ => {}
                    }
                }
                id = 0usize;
                for sub_section in joint_definition.sub_sections {
                    match sub_section {
                        JointDefinitionSubSection::Joint(joint) => {
                            let mut i_transform: Option<
                                IPositionCoordinates,
                            > = None;
                            let mut rotation_euler: Option<(
                                f32,
                                f32,
                                f32,
                            )> = None;
                            for sub_section in joint.sub_sections {
                                match sub_section {
                                    JointSubSection::Offsets(offsets) => {
                                        for attribute in offsets.attributes {
                                            match attribute {
                                                JointAttribute::Transform(t) => {
                                                    i_transform = Some(IPositionCoordinates {
                                                        x: t.x,
                                                        y: t.y,
                                                        z: t.z,
                                                    });
                                                }
                                                JointAttribute::EulerRoll(r) => {
                                                    match rotation_euler {
                                                        Some(ref mut h) => {
                                                            h.0 = r.angle * DEG_TO_RAD
                                                        }
                                                        None => {
                                                            rotation_euler = Some((
                                                                r.angle * DEG_TO_RAD,
                                                                0f32,
                                                                0f32,
                                                            ))
                                                        }
                                                    }
                                                }
                                                JointAttribute::EulerPitch(p) => {
                                                    match rotation_euler {
                                                        Some(ref mut h) => {
                                                            h.1 = p.angle * DEG_TO_RAD
                                                        }
                                                        None => {
                                                            rotation_euler = Some((
                                                                0f32,
                                                                p.angle * DEG_TO_RAD,
                                                                0f32,
                                                            ))
                                                        }
                                                    }
                                                }
                                                JointAttribute::EulerYaw(y) => match rotation_euler
                                                {
                                                    Some(ref mut h) => h.2 = y.angle * DEG_TO_RAD,
                                                    None => {
                                                        rotation_euler =
                                                            Some((0f32, 0f32, y.angle * DEG_TO_RAD))
                                                    }
                                                },
                                                _ => {}
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            i_nodes.push(INode {
                                transform: i_transform,
                                rotation: match rotation_euler {
                                    Some(euler) => {
                                        let yaw = euler.2;
                                        let roll = euler.0;
                                        let pitch = euler.1;
                                        let cy = (yaw * 0.5).cos();
                                        let sy = (yaw * 0.5).sin();
                                        let cr = (roll * 0.5).cos();
                                        let sr = (roll * 0.5).sin();
                                        let cp = (pitch * 0.5).cos();
                                        let sp = (pitch * 0.5).sin();
                                        let w = cy * cr * cp + sy * sr * sp;
                                        let x = cy * sr * cp - sy * cr * sp;
                                        let y = cy * cr * sp + sy * sr * cp;
                                        let z = sy * cr * cp - cy * sr * sp;
                                        Some(IQuaternion { x, y, z, w })
                                    }
                                    None => None,
                                },
                                name: joint.name,
                                children: i_children[id].clone(),
                            });
                            id += 1;
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    // Create glTF sections
    let mut scenes = Vec::new();
    let mut nodes = Vec::new();
    let mut buffers = Vec::new();
    let mut buffer_views = Vec::new();
    let mut accessors = Vec::new();
    let mut meshes = Vec::new();

    // Write binary files and add data to glTF
    // First, write and add vertices
    let mut i_position_extremes = IVerticesExtremes::new(
        i_vertices[0].position_coordinates.x,
        i_vertices[0].position_coordinates.y,
        i_vertices[0].position_coordinates.z,
    );
    let mut vertices_positions_file =
        File::create(output_path.join("vertices_positions.bin")).unwrap();
    let mut vertices_uv_maps_file = File::create(output_path.join("vertices_uv_maps.bin")).unwrap();
    for i_vertex in i_vertices.clone() {
        i_position_extremes
            .x
            .update(i_vertex.position_coordinates.x);
        i_position_extremes
            .y
            .update(i_vertex.position_coordinates.y);
        i_position_extremes
            .z
            .update(i_vertex.position_coordinates.z);
        vertices_positions_file
            .write_le_to_f32(i_vertex.position_coordinates.x)
            .unwrap();
        vertices_positions_file
            .write_le_to_f32(i_vertex.position_coordinates.y)
            .unwrap();
        vertices_positions_file
            .write_le_to_f32(i_vertex.position_coordinates.z)
            .unwrap();
        vertices_uv_maps_file
            .write_le_to_f32(f32::from(i_vertex.texture_coordinates.u))
            .unwrap();
        vertices_uv_maps_file
            .write_le_to_f32(f32::from(i_vertex.texture_coordinates.v))
            .unwrap();
    }
    // Positions
    let buffer_id = buffers.len();
    buffers.push(Buffer {
        byte_length: i_vertices.len() * 3 * 4,
        uri: Some("vertices_positions.bin".to_owned()),
    });
    let buffer_view_id = buffer_views.len();
    buffer_views.push(BufferView {
        buffer: buffer_id,
        byte_offset: None,
        byte_length: i_vertices.len() * 3 * 4,
        byte_stride: None,
    });
    let vertices_positions_accessor_id = accessors.len();
    let min_max = i_position_extremes.to_vec();
    accessors.push(Accessor {
        buffer_view: Some(buffer_view_id),
        component_type: 5126,
        count: i_vertices.len(),
        attribute_type: "VEC3".to_owned(),
        min: Some(min_max[0].clone()),
        max: Some(min_max[1].clone()),
    });
    // UV Maps
    let buffer_id = buffers.len();
    buffers.push(Buffer {
        byte_length: i_vertices.len() * 4 * 2,
        uri: Some("vertices_uv_maps.bin".to_owned()),
    });
    let buffer_view_id = buffer_views.len();
    buffer_views.push(BufferView {
        buffer: buffer_id,
        byte_offset: None,
        byte_length: i_vertices.len() * 4 * 2,
        byte_stride: None,
    });
    let vertices_uv_maps_accessor_id = accessors.len();
    accessors.push(Accessor {
        buffer_view: Some(buffer_view_id),
        component_type: 5126,
        count: i_vertices.len(),
        attribute_type: "VEC2".to_owned(),
        min: None,
        max: None,
    });

    // Next, Meshes
    let mut scene_nodes = Vec::new();
    let mut mesh_counter = 0usize;
    for i_mesh in i_meshes {
        let mut shape_file =
            File::create(output_path.join(format!("mesh_{}.bin", mesh_counter))).unwrap();
        for i_face in i_mesh.faces.clone() {
            shape_file.write_le_to_u16(i_face.0).unwrap();
            shape_file.write_le_to_u16(i_face.1).unwrap();
            shape_file.write_le_to_u16(i_face.2).unwrap();
        }
        scene_nodes.push(nodes.len());
        nodes.push(Node {
            mesh: Some(mesh_counter),
            children: None,
            translation: None,
            name: None,
            skin: None,
            rotation: None,
            scale: None,
        });
        let buffer_id = buffers.len();
        buffers.push(Buffer {
            byte_length: i_mesh.faces.len() * 2 * 3,
            uri: Some(format!("mesh_{}.bin", mesh_counter)),
        });
        let buffer_view_id = buffer_views.len();
        buffer_views.push(BufferView {
            buffer: buffer_id,
            byte_offset: None,
            byte_length: i_mesh.faces.len() * 2 * 3,
            byte_stride: None,
        });
        let accessor_id = accessors.len();
        accessors.push(Accessor {
            buffer_view: Some(buffer_view_id),
            component_type: 5123,
            count: i_mesh.faces.len() * 3,
            attribute_type: "SCALAR".to_owned(),
            max: None,
            min: None,
        });
        meshes.push(Mesh {
            primitives: {
                let mut primitives = Vec::new();
                primitives.push(Primitive {
                    attributes: {
                        let mut map = HashMap::new();
                        map.insert("POSITION".to_owned(), vertices_positions_accessor_id);
                        map.insert("TEXCOORD_0".to_owned(), vertices_uv_maps_accessor_id);
                        map
                    },
                    indices: Some(accessor_id),
                    material: None,
                });
                primitives
            },
        });
        mesh_counter += 1;
    }

    // Then, Joints
    let start = nodes.len();
    for (i, i_node) in i_nodes.into_iter().enumerate() {
        if i_root_nodes.contains(&i) {
            scene_nodes.push(nodes.len());
        }
        let ch = if !i_node.children.is_empty() {
            let mut fixed = Vec::new();
            for id in i_node.children {
                fixed.push(id + start);
            }
            Some(fixed)
        } else {
            None
        };
        nodes.push(Node {
            mesh: None,
            translation: match i_node.transform {
                Some(t) => Some([t.x, t.y, t.z]),
                None => None,
            },
            name: Some(i_node.name),
            rotation: match i_node.rotation {
                Some(r) => Some([r.x, r.y, r.z, r.w]),
                None => None,
            },
            scale: None,
            children: ch,
            skin: None,
        });
    }

    scenes.push(Scene {
        nodes: Some(scene_nodes),
    });

    // Export glTF
    let gltf = GlTF {
        asset: Asset {
            version: "2.0".to_owned(),
        },
        scene: Some(0),
        scenes: Some(scenes),
        nodes: Some(nodes),
        buffers: Some(buffers),
        buffer_views: Some(buffer_views),
        accessors: Some(accessors),
        meshes: Some(meshes),
        skins: None,
        materials: None,
        textures: None,
        images: None,
    };
    let mut gltf_out =
        File::create(output_path.join("model.gltf")).expect("Impossible to create file");
    gltf.write_gltf_pretty(&mut gltf_out).unwrap();
}
