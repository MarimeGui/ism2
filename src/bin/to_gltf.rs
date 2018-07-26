extern crate clap;
extern crate ez_io;
extern crate ism2;
extern crate my_gltf;

use clap::{App, Arg};
use ez_io::WriteE;
use ism2::{
    joint_definition::JointAttribute, joint_definition::JointDefinitionSubSection,
    joint_definition::JointSubSection, model_data::ShapeSubSection, model_data::SubSection,
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
    let reader = &mut BufReader::new(File::open(input_path).unwrap());
    let ism = ISM2::import(reader).unwrap();

    // Create vectors holding glTF information
    let mut scenes = Vec::new();
    let mut nodes = Vec::new();
    let mut buffers = Vec::new();
    let mut buffer_views = Vec::new();
    let mut accessors = Vec::new();
    let mut meshes = Vec::new();

    // Sections
    let mut scene_nodes = Vec::new();
    for section in ism.sections {
        match section {
            // Model Data
            Section::ModelData(model_data) => {
                let mut shape_counter = 0usize;
                let mut vertices_positions_accessor_id: usize = 0usize;
                let mut vertices_uv_accessor_id: usize = 0usize;
                for sub_section in model_data.zero_a.sub_sections {
                    match sub_section {
                        // Vertices
                        SubSection::Vertices(vertices) => {
                            for vertices_sub_section in vertices.sub_sections {
                                match vertices_sub_section {
                                    // Vertices Data Buffer
                                    VerticesSubSection::Data(data) => {
                                        // Create the position file, the min and max variable and the size variable
                                        let mut vertices_positions_file =
                                            File::create(
                                                output_path.join("vertices_positions.bin"),
                                            ).unwrap();
                                        let mut vertices_positions_min_max: ((f32, f32), (f32, f32), (f32, f32)) = (
                                            (
                                                data.vertices[0].position_coordinates[0],
                                                data.vertices[0].position_coordinates[0],
                                            ),
                                            (
                                                data.vertices[0].position_coordinates[1],
                                                data.vertices[0].position_coordinates[1],
                                            ),
                                            (
                                                data.vertices[0].position_coordinates[2],
                                                data.vertices[0].position_coordinates[2],
                                            ),
                                        ); // X, Y, Z -> Min, Max
                                        let mut vertices_positions_file_len = 0usize;
                                        // Create the UV Maps file and the size counter
                                        let mut vertices_uv_maps_file =
                                            File::create(output_path.join("vertices_uv_maps.bin"))
                                                .unwrap();
                                        let mut vertices_uv_maps_len = 0usize;
                                        // For each vertex
                                        for vertex in data.vertices {
                                            // Write position
                                            vertices_positions_file
                                                .write_le_to_f32(vertex.position_coordinates[0])
                                                .unwrap();
                                            vertices_positions_file
                                                .write_le_to_f32(vertex.position_coordinates[1])
                                                .unwrap();
                                            vertices_positions_file
                                                .write_le_to_f32(vertex.position_coordinates[2])
                                                .unwrap();
                                            vertices_positions_file_len += 12;
                                            // Get Min and Max
                                            if (vertices_positions_min_max.0)
                                                .0
                                                .gt(&vertex.position_coordinates[0])
                                            {
                                                // X Min
                                                (vertices_positions_min_max.0).0 =
                                                    vertex.position_coordinates[0]
                                            }
                                            if (vertices_positions_min_max.1)
                                                .0
                                                .gt(&vertex.position_coordinates[1])
                                            {
                                                // Y Min
                                                (vertices_positions_min_max.1).0 =
                                                    vertex.position_coordinates[1]
                                            }
                                            if (vertices_positions_min_max.2)
                                                .0
                                                .gt(&vertex.position_coordinates[2])
                                            {
                                                // Z Min
                                                (vertices_positions_min_max.2).0 =
                                                    vertex.position_coordinates[2]
                                            }
                                            if (vertices_positions_min_max.0)
                                                .1
                                                .lt(&vertex.position_coordinates[0])
                                            {
                                                // X Max
                                                (vertices_positions_min_max.0).1 =
                                                    vertex.position_coordinates[0]
                                            }
                                            if (vertices_positions_min_max.1)
                                                .1
                                                .lt(&vertex.position_coordinates[1])
                                            {
                                                // Y Max
                                                (vertices_positions_min_max.1).1 =
                                                    vertex.position_coordinates[1]
                                            }
                                            if (vertices_positions_min_max.2)
                                                .1
                                                .lt(&vertex.position_coordinates[2])
                                            {
                                                // Z Max
                                                (vertices_positions_min_max.2).1 =
                                                    vertex.position_coordinates[2]
                                            }
                                            // Write UV Maps
                                            vertices_uv_maps_file
                                                .write_le_to_f32(f32::from(
                                                    vertex.texture_coordinates[0],
                                                ))
                                                .unwrap();
                                            vertices_uv_maps_file
                                                .write_le_to_f32(f32::from(
                                                    vertex.texture_coordinates[1],
                                                ))
                                                .unwrap();
                                            vertices_uv_maps_len += 8;
                                        }
                                        // Add Positions to glTF
                                        let buffer_id = buffers.len();
                                        buffers.push(Buffer {
                                            byte_length: vertices_positions_file_len,
                                            uri: Some("vertices_positions.bin".to_owned()),
                                        });
                                        let buffer_view_id = buffer_views.len();
                                        buffer_views.push(BufferView {
                                            buffer: buffer_id,
                                            byte_offset: None,
                                            byte_length: vertices_positions_file_len,
                                            byte_stride: None,
                                        });
                                        vertices_positions_accessor_id = accessors.len();
                                        accessors.push(Accessor {
                                            buffer_view: Some(buffer_view_id),
                                            component_type: 5126,
                                            count: vertices.nb_vertices as usize,
                                            attribute_type: "VEC3".to_owned(),
                                            min: Some(vec![
                                                (vertices_positions_min_max.0).0,
                                                (vertices_positions_min_max.1).0,
                                                (vertices_positions_min_max.2).0,
                                            ]),
                                            max: Some(vec![
                                                (vertices_positions_min_max.0).1,
                                                (vertices_positions_min_max.1).1,
                                                (vertices_positions_min_max.2).1,
                                            ]),
                                        });
                                        // Add UV Maps to glTF
                                        let buffer_id = buffers.len();
                                        buffers.push(Buffer {
                                            byte_length: vertices_uv_maps_len,
                                            uri: Some("vertices_uv_maps.bin".to_owned()),
                                        });
                                        let buffer_view_id = buffer_views.len();
                                        buffer_views.push(BufferView {
                                            buffer: buffer_id,
                                            byte_offset: None,
                                            byte_length: vertices_uv_maps_len,
                                            byte_stride: None,
                                        });
                                        vertices_uv_accessor_id = accessors.len();
                                        accessors.push(Accessor {
                                            buffer_view: Some(buffer_view_id),
                                            component_type: 5126,
                                            count: vertices.nb_vertices as usize,
                                            attribute_type: "VEC2".to_owned(),
                                            min: None,
                                            max: None,
                                        });
                                    }
                                    _ => {}
                                }
                            }
                        }

                        // Shape
                        SubSection::Shape(faces) => {
                            // Create the file and size counter
                            let mut shape_file = File::create(
                                output_path.join(format!("shape_{}.bin", shape_counter)),
                            ).unwrap();
                            let mut shape_file_size = 0usize;
                            for sub_section in faces.sub_sections {
                                match sub_section {
                                    ShapeSubSection::Faces(data) => {
                                        // Write each face to the file
                                        for face in data.faces {
                                            shape_file.write_le_to_u16(face.points.0).unwrap();
                                            shape_file.write_le_to_u16(face.points.1).unwrap();
                                            shape_file.write_le_to_u16(face.points.2).unwrap();
                                            shape_file_size += 6;
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            // Add node with this model as an attribute, also add it to the list of nodes that composes the scene
                            scene_nodes.push(nodes.len());
                            nodes.push(Node {
                                mesh: Some(shape_counter),
                                children: None,
                                translation: None,
                                name: None,
                                skin: None,
                                rotation: None,
                                scale: None,
                            });
                            // Add the shape to the glTF
                            let buffer_id = buffers.len();
                            buffers.push(Buffer {
                                byte_length: shape_file_size,
                                uri: Some(format!("shape_{}.bin", shape_counter)),
                            });
                            let buffer_view_id = buffer_views.len();
                            buffer_views.push(BufferView {
                                buffer: buffer_id,
                                byte_offset: None,
                                byte_length: shape_file_size,
                                byte_stride: None,
                            });
                            let accessor_id = accessors.len();
                            accessors.push(Accessor {
                                buffer_view: Some(buffer_view_id),
                                component_type: 5123,
                                count: faces.nb_faces as usize * 3,
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
                                            map.insert(
                                                "POSITION".to_owned(),
                                                vertices_positions_accessor_id,
                                            );
                                            map.insert(
                                                "TEXCOORD_0".to_owned(),
                                                vertices_uv_accessor_id,
                                            );
                                            map
                                        },
                                        indices: Some(accessor_id),
                                        material: None,
                                    });
                                    primitives
                                },
                            });
                            shape_counter += 1;
                        }

                        // Not handling everything for now
                        _ => {}
                    }
                }
                // Add the scene to the glTF
            }
            // Joint Definitions
            Section::JointDefinition(joint_definition) => {
                let mut joints_name: Vec<String> = Vec::new();
                let mut joints_translation: Vec<Option<[f32; 3]>> = Vec::new();
                let mut joints_rotation: Vec<Option<[f32; 4]>> = Vec::new();
                let mut children: Vec<Vec<usize>> = Vec::new();
                let mut root_nodes: Vec<usize> = Vec::new();
                let mut id = 0usize;
                for sub_section in joint_definition.sub_sections {
                    match sub_section {
                        JointDefinitionSubSection::Joint(joint) => {
                            joints_name.push(joint.name);
                            children.push(Vec::new());
                            match joint.parent_index {
                                None => {
                                    root_nodes.push(id)
                                },
                                Some(p) => match children.get_mut(p) {
                                    Some(ref mut c) => {
                                        c.push(id)
                                    },
                                    None => panic!("Missing parent"),
                                },
                            }
                            for sub_section in joint.sub_sections {
                                match sub_section {
                                    JointSubSection::Offsets(offsets) => {
                                        let mut transform = None;
                                        let mut rotation_euler: Option<[f32; 3]> = None;
                                        for attribute in offsets.attributes {
                                            match attribute {
                                                JointAttribute::Transform(t) => {
                                                    transform = Some([t.x, t.y, t.z]);
                                                }
                                                JointAttribute::EulerRoll(r) => {
                                                    match rotation_euler {
                                                        Some(ref mut h) => {
                                                            h[0] = r.angle * DEG_TO_RAD
                                                        }
                                                        None => {
                                                            rotation_euler = Some([
                                                                r.angle * DEG_TO_RAD,
                                                                0f32,
                                                                0f32,
                                                            ])
                                                        }
                                                    }
                                                }
                                                JointAttribute::EulerPitch(p) => {
                                                    match rotation_euler {
                                                        Some(ref mut h) => {
                                                            h[1] = p.angle * DEG_TO_RAD
                                                        }
                                                        None => {
                                                            rotation_euler = Some([
                                                                0f32,
                                                                p.angle * DEG_TO_RAD,
                                                                0f32,
                                                            ])
                                                        }
                                                    }
                                                }
                                                JointAttribute::EulerYaw(y) => match rotation_euler
                                                {
                                                    Some(ref mut h) => h[2] = y.angle * DEG_TO_RAD,
                                                    None => {
                                                        rotation_euler =
                                                            Some([0f32, 0f32, y.angle * DEG_TO_RAD])
                                                    }
                                                },
                                                _ => {}
                                            }
                                        }
                                        joints_translation.push(transform);
                                        match rotation_euler {
                                            None => joints_rotation.push(None),
                                            Some(r) => {
                                                let yaw = r[2];
                                                let roll = r[0];
                                                let pitch = r[1];
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
                                                let rotation_quaternion = [x, y, z, w];
                                                joints_rotation.push(Some(rotation_quaternion));
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            id += 1;
                        }
                        _ => {}
                    }
                }
                // let mut joint_ids = Vec::new();
                let start = nodes.len();
                for i in 0..joints_name.len() {
                    if root_nodes.contains(&i) {
                        scene_nodes.push(nodes.len());
                    }
                    let ch = match children.get(i) {
                        Some(c) => {
                            if !c.is_empty() {
                                let mut fixed = Vec::new();
                                for id in c {
                                    fixed.push(id + start);
                                }
                                Some(fixed)
                            } else {
                                None
                            }
                        }
                        None => panic!(),
                    };
                    // joint_ids.push(counter + i);
                    nodes.push(Node {
                        mesh: None,
                        translation: match joints_translation.get(i) {
                            Some(t) => t.clone(),
                            None => panic!(),
                        },
                        name: match joints_name.get(i) {
                            Some(n) => Some(n.clone()),
                            None => panic!(),
                        },
                        rotation: match joints_rotation.get(i) {
                            Some(r) => r.clone(),
                            None => panic!(),
                        },
                        scale: None,
                        children: ch,
                        skin: None,
                    });
                }
            }
            _ => {}
        }
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

    // Write glTF down
    let mut gltf_out =
        File::create(output_path.join("model.gltf")).expect("Impossible to create file");
    gltf.write_gltf_pretty(&mut gltf_out).unwrap();
}
