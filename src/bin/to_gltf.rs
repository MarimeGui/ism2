extern crate clap;
extern crate ez_io;
extern crate half;
extern crate ism2;
extern crate my_gltf;
extern crate png;
extern crate rgb;
extern crate tid;

use clap::{App, Arg};
use ez_io::WriteE;
use half::f16;
use ism2::{
    joint_definition::JointAttribute, joint_definition::JointDefinitionSubSection,
    joint_definition::JointSubSection, joint_extra::BufferData, model_data::FrenetFrame,
    model_data::MeshSubSection, model_data::SubSection, model_data::VerticesDataBuffer, Section,
    ISM2,
};
use my_gltf::{
    accessors::Accessor, asset::Asset, buffer_views::BufferView, buffers::Buffer, images::Image,
    materials::BaseColorTexture, materials::Material, materials::PbrMetallicRoughness,
    meshes::Mesh, meshes::Primitive, nodes::Node, scenes::Scene, skins::Skin, textures::Texture,
    GlTF,
};
use png::{Encoder, HasParameters};
use rgb::ComponentBytes;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::f32::consts::PI;
use std::fs::{create_dir_all, File};
use std::io::ErrorKind as IOErrorKind;
use std::io::{BufReader, BufWriter};
use std::io::{Seek, SeekFrom};
use std::path::Path;
use std::process::exit;
use tid::TID;

const DEG_TO_RAD: f32 = PI / 180f32;

#[derive(Clone)]
struct IVertex {
    position_coordinates: IPositionCoordinates,
    texture_coordinates: ITextureCoordinates,
    frenet_frame: IFrenetFrame,
}

#[derive(Clone)]
struct IVertexRig {
    joints: (u8, u8, u8, u8),
    weights: (f32, f32, f32, f32),
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
struct IFrenetFrame {
    normal: IVector,
    tangent: IVector,
}

#[derive(Clone)]
struct IVector {
    x: f32,
    y: f32,
    z: f32,
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
    scale: Option<IPositionCoordinates>,
    name: String,
    children: Vec<usize>,
}

#[derive(Clone)]
struct IEulerAngle {
    x: f32,
    y: f32,
    z: f32,
    rotation_order: IEulerRotationOrder,
}

#[derive(Clone)]
struct IQuaternion {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}

#[derive(Clone)]
enum IEulerRotationOrder {
    // XYZ,
    // XZY,
    // YXZ,
    // YZX,
    // ZXY,
    ZYX,
}

#[derive(Clone)]
enum IParity {
    // Even,
    Odd,
}

#[derive(Clone)]
struct IJoint {
    matrix: [f32; 16],
}

impl IFrenetFrame {
    fn import(frame: &FrenetFrame<f16>) -> IFrenetFrame {
        let normal_x = f32::from(frame.normal.x);
        let normal_y = f32::from(frame.normal.y);
        let normal_z = f32::from(frame.normal.z);
        let normal_length = (normal_x.powi(2) + normal_y.powi(2) + normal_z.powi(2)).sqrt();
        let normal_coefficient = if normal_length.is_normal() {
            1.0 / normal_length
        } else {
            println!("/!\\ Wrong Normal Length {}", normal_length);
            0.0
        };
        let tangent_x = f32::from(frame.tangent.x);
        let tangent_y = f32::from(frame.tangent.y);
        let tangent_z = f32::from(frame.tangent.z);
        let tangent_length = (tangent_x.powi(2) + tangent_y.powi(2) + tangent_z.powi(2)).sqrt();
        let tangent_coefficient;
        if tangent_length.is_normal() {
            tangent_coefficient =
                1.0 / (tangent_x.powi(2) + tangent_y.powi(2) + tangent_z.powi(2)).sqrt();
        } else {
            // println!("/!\\ Wrong Tangent Length {}", normal_length);
            tangent_coefficient = 0.0;
        }
        IFrenetFrame {
            normal: IVector {
                x: normal_x * normal_coefficient,
                y: normal_y * normal_coefficient,
                z: normal_z * normal_coefficient,
            },
            tangent: IVector {
                x: tangent_x * tangent_coefficient,
                y: tangent_y * tangent_coefficient,
                z: tangent_z * tangent_coefficient,
            },
        }
    }
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

impl IEulerAngle {
    fn get_angle_from_index(&self, i: usize) -> f32 {
        match i {
            0 => self.x,
            1 => self.y,
            2 => self.z,
            x => panic!("{} out of range", x), // Meh
        }
    }
}

// This code mostly comes from Blender's source code, as well as some other parts of the conversion from Euler Angles to Quaternions.
impl IQuaternion {
    fn from_euler(euler: IEulerAngle) -> IQuaternion {
        // We're supposed to use parity, but it works better without it...
        let i = euler.rotation_order.get_info().0 .0;
        let j = euler.rotation_order.get_info().0 .1;
        let k = euler.rotation_order.get_info().0 .2;
        let mut a = [0f32; 3];
        let mut q = [0f32; 4];

        let ti = euler.get_angle_from_index(i) * 0.5;
        let tj = euler.get_angle_from_index(j) // * match euler.rotation_order.get_info().1 {
        //     IParity::Even => 0.5,
        //     IParity::Odd => -0.5,
        // };
        * 0.5;
        let th = euler.get_angle_from_index(k) * 0.5;

        let ci = ti.cos();
        let cj = tj.cos();
        let ch = th.cos();
        let si = ti.sin();
        let sj = tj.sin();
        let sh = th.sin();

        let cc = ci * ch;
        let cs = ci * sh;
        let sc = si * ch;
        let ss = si * sh;

        a[i] = cj * sc - sj * cs;
        a[j] = cj * ss + sj * cc;
        a[k] = cj * cs - sj * sc;

        q[0] = cj * cc + sj * ss; // W
        q[1] = a[0]; // X
        q[2] = a[1]; // Y
        q[3] = a[2]; // Z

        // match euler.rotation_order.get_info().1 {
        //     IParity::Odd => {q[j+1] = -q[j+1]}
        //     IParity::Even => {}
        // }

        IQuaternion {
            x: q[1],
            y: q[2],
            z: q[3],
            w: q[0],
        }
    }
}

impl IEulerRotationOrder {
    fn get_info(&self) -> ((usize, usize, usize), IParity) {
        match self {
            // IEulerRotationOrder::XYZ => ((0, 1, 2), IParity::Even),
            // IEulerRotationOrder::XZY => ((0, 2, 1), IParity::Odd),
            // IEulerRotationOrder::YXZ => ((1, 0, 2), IParity::Odd),
            // IEulerRotationOrder::YZX => ((1, 2, 0), IParity::Even),
            // IEulerRotationOrder::ZXY => ((2, 0, 1), IParity::Even),
            IEulerRotationOrder::ZYX => ((2, 1, 0), IParity::Odd),
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
        ).arg(
            Arg::with_name("OUTPUT")
                .help("Folder name for output")
                .required(true)
                .index(2),
        ).get_matches();

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
    let mut i_vertices_rig: Vec<IVertexRig> = Vec::new();
    let mut i_meshes: Vec<IMesh> = Vec::new();
    let mut i_nodes: Vec<INode> = Vec::new();
    let mut i_root_nodes: Vec<usize> = Vec::new();
    let mut i_joints: Vec<IJoint> = Vec::new();
    let mut i_textures: Vec<String> = Vec::new();
    let mut i_in_vertex_id_to_joint_id: HashMap<u32, usize> = HashMap::new();

    // Get the required information from the ISM file
    for section in ism.sections {
        match section {
            Section::ModelData(model_data) => {
                for sub_section in model_data.zero_a.sub_sections {
                    match sub_section {
                        SubSection::Vertices(vertices) => match vertices.buffer {
                            VerticesDataBuffer::Geometry(g) => {
                                for vertex in g.vertices {
                                    i_vertices.push(IVertex {
                                        position_coordinates: IPositionCoordinates {
                                            x: vertex.position_coordinates.x,
                                            y: vertex.position_coordinates.y,
                                            z: vertex.position_coordinates.z,
                                        },
                                        texture_coordinates: ITextureCoordinates {
                                            u: vertex.texture_coordinates.u,
                                            v: vertex.texture_coordinates.v,
                                        },
                                        frenet_frame: IFrenetFrame::import(&vertex.frenet_frame),
                                    });
                                }
                            }
                            VerticesDataBuffer::Rigging(r) => {
                                for vertex in r.vertices {
                                    i_vertices_rig.push(IVertexRig {
                                        joints: vertex.joints,
                                        weights: vertex.weights,
                                    });
                                }
                            }
                        },
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
                            i_in_vertex_id_to_joint_id.insert(joint.in_vertex_id, id);
                            let mut i_transform: Option<
                                IPositionCoordinates,
                            > = None;
                            let mut rotation_euler: Option<(
                                f32,
                                f32,
                                f32,
                            )> = None;
                            let mut scale: Option<IPositionCoordinates> = None;
                            for sub_section in joint.sub_sections {
                                match sub_section {
                                    JointSubSection::Offsets(offsets) => {
                                        for attribute in offsets.attributes {
                                            match attribute {
                                                JointAttribute::Translate(t) => {
                                                    i_transform = Some(IPositionCoordinates {
                                                        x: t.x,
                                                        y: t.y,
                                                        z: t.z,
                                                    });
                                                }
                                                JointAttribute::Scale(s) => {
                                                    scale = Some(IPositionCoordinates {
                                                        x: s.x,
                                                        y: s.y,
                                                        z: s.z,
                                                    })
                                                }
                                                JointAttribute::JointOrientX(r) => {
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
                                                JointAttribute::JointOrientY(p) => {
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
                                                JointAttribute::JointOrientZ(y) => {
                                                    match rotation_euler {
                                                        Some(ref mut h) => {
                                                            h.2 = y.angle * DEG_TO_RAD
                                                        }
                                                        None => {
                                                            rotation_euler = Some((
                                                                0f32,
                                                                0f32,
                                                                y.angle * DEG_TO_RAD,
                                                            ))
                                                        }
                                                    }
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            i_nodes.push(INode {
                                transform: i_transform,
                                scale,
                                rotation: match rotation_euler {
                                    Some(euler) => {
                                        let euler_angle = IEulerAngle {
                                            x: euler.0,
                                            y: euler.1,
                                            z: euler.2,
                                            rotation_order: IEulerRotationOrder::ZYX, // Seems to be the case everywhere
                                        };

                                        Some(IQuaternion::from_euler(euler_angle))
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
            Section::JointExtra(je) => {
                for s1 in je.sub_sections {
                    for s2 in s1.sub_sections {
                        for s3 in s2.sub_sections {
                            match s3.data {
                                BufferData::BoneNames(_) => {}
                                BufferData::InverseBindMatrices(inv) => {
                                    for matrix in inv {
                                        i_joints.push(IJoint { matrix });
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Section::TextureDefinition(td) => {
                for texture in td.sub_sections {
                    i_textures.push(texture.base_name);
                }
            }
        }
    }

    // Create glTF sections
    let mut scenes = Vec::new();
    let mut nodes = Vec::new();
    let mut buffers = Vec::new();
    let mut buffer_views = Vec::new();
    let mut accessors = Vec::new();
    let mut meshes = Vec::new();
    let mut materials = Vec::new();
    let mut skins = Vec::new();
    let mut images = Vec::new();
    let mut textures = Vec::new();

    // Write binary files
    let mut i_position_extremes = IVerticesExtremes::new(
        i_vertices[0].position_coordinates.x,
        i_vertices[0].position_coordinates.y,
        i_vertices[0].position_coordinates.z,
    );
    let mut vertices_positions_file =
        File::create(output_path.join("vertices_positions.bin")).unwrap();
    let mut vertices_uv_maps_file = File::create(output_path.join("vertices_uv_maps.bin")).unwrap();
    let mut vertices_normals_file = File::create(output_path.join("vertices_normals.bin")).unwrap();
    // let mut vertices_tangents_file =
    //     File::create(output_path.join("vertices_tangents.bin")).unwrap();
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
        vertices_normals_file
            .write_le_to_f32(i_vertex.frenet_frame.normal.x)
            .unwrap();
        vertices_normals_file
            .write_le_to_f32(i_vertex.frenet_frame.normal.y)
            .unwrap();
        vertices_normals_file
            .write_le_to_f32(i_vertex.frenet_frame.normal.z)
            .unwrap();
        // Disabled Tangents as I cannot confirm if the data read from the file are actually Tangents
        // vertices_tangents_file
        //     .write_le_to_f32(0.0) // .write_le_to_f32(i_vertex.frenet_frame.tangent.x)
        //     .unwrap();
        // vertices_tangents_file
        //     .write_le_to_f32(1.0) // .write_le_to_f32(i_vertex.frenet_frame.tangent.y)
        //     .unwrap();
        // vertices_tangents_file
        //     .write_le_to_f32(0.0) // .write_le_to_f32(i_vertex.frenet_frame.tangent.z)
        //     .unwrap();
        // vertices_tangents_file
        //     .write_le_to_f32(1.0)  // Handedness
        //     .unwrap();
    }
    if !i_joints.is_empty() {
        let mut vertices_inv_file = File::create(output_path.join("joints_inv.bin")).unwrap();
        for (i, i_joint) in i_joints.clone().iter().enumerate() {
            let write_to = i_in_vertex_id_to_joint_id.get(&(i as u32)).unwrap() * 64;
            vertices_inv_file
                .seek(SeekFrom::Start(write_to as u64))
                .unwrap(); // Lossy
            for id in vec![0usize, 4, 8, 12, 1, 5, 9, 13, 2, 6, 10, 14, 3, 7, 11, 15] {
                // This is weird
                vertices_inv_file
                    .write_le_to_f32(i_joint.matrix[id])
                    .unwrap();
            }
        }
    }
    if !i_vertices_rig.is_empty() {
        let mut vertices_joints = File::create(output_path.join("vertices_joints.bin")).unwrap();
        let mut vertices_weights = File::create(output_path.join("vertices_weights.bin")).unwrap();
        for i_vertex_rig in i_vertices_rig.clone() {
            // vertices_joints.write_to_u8(i_vertex_rig.joints.0).unwrap();
            vertices_joints
                .write_to_u8(
                    *i_in_vertex_id_to_joint_id
                        .get(&u32::from(i_vertex_rig.joints.0))
                        .unwrap() as u8,
                ).unwrap(); // Lossy
            vertices_joints
                .write_to_u8(
                    *i_in_vertex_id_to_joint_id
                        .get(&u32::from(i_vertex_rig.joints.1))
                        .unwrap() as u8,
                ).unwrap(); // Lossy
            vertices_joints
                .write_to_u8(
                    *i_in_vertex_id_to_joint_id
                        .get(&u32::from(i_vertex_rig.joints.2))
                        .unwrap() as u8,
                ).unwrap(); // Lossy
            vertices_joints
                .write_to_u8(
                    *i_in_vertex_id_to_joint_id
                        .get(&u32::from(i_vertex_rig.joints.3))
                        .unwrap() as u8,
                ).unwrap(); // Lossy
            vertices_weights
                .write_le_to_f32(i_vertex_rig.weights.0)
                .unwrap();
            vertices_weights
                .write_le_to_f32(i_vertex_rig.weights.1)
                .unwrap();
            vertices_weights
                .write_le_to_f32(i_vertex_rig.weights.2)
                .unwrap();
            vertices_weights
                .write_le_to_f32(i_vertex_rig.weights.3)
                .unwrap();
        }
    }
    for tex_name in i_textures {
        // Sometimes, the ISM2 file does not specify the correct name as the first parameter, this is a quick fix that will work for most files...
        // println!("Tex name: {}", tex_name);
        let in_tid = {
            let mut file = match File::open(
                input_path
                    .parent()
                    .unwrap()
                    .join(format!("texture/001/{}.tid", tex_name)),
            ) {
                Ok(f) => f,
                Err(e) => if e.kind() == IOErrorKind::NotFound {
                    match File::open(
                        input_path
                            .parent()
                            .unwrap()
                            .join(format!("texture/001/tex_c.tid")),
                    ) {
                        Ok(f) => f,
                        Err(e) => {
                            if e.kind() == IOErrorKind::NotFound {
                                println!("/!\\ Failed to open texture '{}.tid', ignoring...", tex_name);
                                continue;
                            } else {
                                panic!(e)
                            }
                        }
                    }
                } else {
                    panic!(e)
                },
            };
            &mut BufReader::new(file)
        };
        let tid = TID::import(in_tid).unwrap();
        let image = tid.convert();
        let w = &mut BufWriter::new(
            File::create(output_path.join(format!("{}.png", tex_name))).unwrap(),
        );
        let mut encoder = Encoder::new(w, tid.dimensions.width, tid.dimensions.height);
        encoder.set(png::ColorType::RGBA).set(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(image.as_bytes()).unwrap();
        let image_id = images.len();
        images.push(Image {
            uri: Some(format!("{}.png", tex_name)),
            buffer_view: None,
        });
        let texture_id = textures.len();
        textures.push(Texture {
            source: Some(image_id),
            sampler: None,
        });
        materials.push(Material {
            pbr_metallic_roughness: Some(PbrMetallicRoughness {
                base_color_texture: Some(BaseColorTexture {
                    index: Some(texture_id),
                }),
                metallic_factor: Some(0f64),
                roughness_factor: Some(1f64),
            }),
        });
    }

    // Write Buffers, BufferViews and Accessors in glTF file
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

    // Normals
    let buffer_id = buffers.len();
    buffers.push(Buffer {
        byte_length: i_vertices.len() * 4 * 3,
        uri: Some("vertices_normals.bin".to_owned()),
    });
    let buffer_view_id = buffer_views.len();
    buffer_views.push(BufferView {
        buffer: buffer_id,
        byte_offset: None,
        byte_length: i_vertices.len() * 4 * 3,
        byte_stride: None,
    });
    let vertices_normals_accessor_id = accessors.len();
    accessors.push(Accessor {
        buffer_view: Some(buffer_view_id),
        component_type: 5126,
        count: i_vertices.len(),
        attribute_type: "VEC3".to_owned(),
        min: None,
        max: None,
    });

    // Tangents
    // let buffer_id = buffers.len();
    // buffers.push(Buffer {
    //     byte_length: i_vertices.len() * 4 * 4,
    //     uri: Some("vertices_tangents.bin".to_owned()),
    // });
    // let buffer_view_id = buffer_views.len();
    // buffer_views.push(BufferView {
    //     buffer: buffer_id,
    //     byte_offset: None,
    //     byte_length: i_vertices.len() * 4 * 4,
    //     byte_stride: None,
    // });
    // let vertices_tangents_accessor_id = accessors.len();
    // accessors.push(Accessor {
    //     buffer_view: Some(buffer_view_id),
    //     component_type: 5126,
    //     count: i_vertices.len(),
    //     attribute_type: "VEC4".to_owned(),
    //     min: None,
    //     max: None,
    // });

    // Joints and Weights if necessary
    let vertices_joints_accessor_id;
    let vertices_weights_accessor_id;
    if !i_vertices_rig.is_empty() {
        // Joints
        let buffer_id = buffers.len();
        buffers.push(Buffer {
            byte_length: i_vertices_rig.len() * 4,
            uri: Some("vertices_joints.bin".to_owned()),
        });
        let buffer_view_id = buffer_views.len();
        buffer_views.push(BufferView {
            buffer: buffer_id,
            byte_offset: None,
            byte_length: i_vertices_rig.len() * 4,
            byte_stride: None,
        });
        vertices_joints_accessor_id = Some(accessors.len());
        accessors.push(Accessor {
            buffer_view: Some(buffer_view_id),
            component_type: 5121,
            count: i_vertices.len(),
            attribute_type: "VEC4".to_owned(),
            min: None,
            max: None,
        });
        // Weights
        let buffer_id = buffers.len();
        buffers.push(Buffer {
            byte_length: i_vertices_rig.len() * 4 * 4,
            uri: Some("vertices_weights.bin".to_owned()),
        });
        let buffer_view_id = buffer_views.len();
        buffer_views.push(BufferView {
            buffer: buffer_id,
            byte_offset: None,
            byte_length: i_vertices_rig.len() * 4 * 4,
            byte_stride: None,
        });
        vertices_weights_accessor_id = Some(accessors.len());
        accessors.push(Accessor {
            buffer_view: Some(buffer_view_id),
            component_type: 5126,
            count: i_vertices.len(),
            attribute_type: "VEC4".to_owned(),
            min: None,
            max: None,
        });
    } else {
        vertices_joints_accessor_id = None;
        vertices_weights_accessor_id = None;
    }

    // Joints Inverse Bind Matrices if necessary
    let joints_inv_accessor_id;
    if !i_joints.is_empty() {
        let buffer_id = buffers.len();
        buffers.push(Buffer {
            byte_length: i_joints.len() * 4 * 4 * 4,
            uri: Some("joints_inv.bin".to_owned()),
        });
        let buffer_view_id = buffer_views.len();
        buffer_views.push(BufferView {
            buffer: buffer_id,
            byte_offset: None,
            byte_length: i_joints.len() * 4 * 4 * 4,
            byte_stride: None,
        });
        joints_inv_accessor_id = Some(accessors.len());
        accessors.push(Accessor {
            buffer_view: Some(buffer_view_id),
            component_type: 5126,
            count: i_joints.len(),
            attribute_type: "MAT4".to_owned(),
            min: None,
            max: None,
        });
    } else {
        joints_inv_accessor_id = None;
    }

    // Write Joints in ISM2 file as nodes
    let start = nodes.len();
    let mut root_node_id = None;
    for (i, i_node) in i_nodes.into_iter().enumerate() {
        // if i_root_nodes.contains(&i) {
        //     scene_nodes.push(nodes.len());
        // }
        let ch = if !i_node.children.is_empty() {
            let mut fixed = Vec::new();
            for id in i_node.children {
                fixed.push(id + start);
            }
            Some(fixed)
        } else {
            None
        };
        if i_node.name == "root" {
            root_node_id = Some(i)
        }
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
            scale: match i_node.scale {
                Some(s) => Some([s.x, s.y, s.z]),
                None => None,
            },
            children: ch,
            skin: None,
        });
    }

    // Create the skin
    if let Some(id) = joints_inv_accessor_id {
        skins.push(Skin {
            inverse_bind_matrices: Some(id),
            skeleton: root_node_id,
            name: None,
            joints: {
                let mut joints = Vec::new();
                for i in 0..nodes.len() {
                    joints.push(i);
                }
                joints
            },
        });
    }

    // Write Meshes to glTF file
    let mut mesh_counter = 0usize;
    let mut mesh_nodes = Vec::new();
    for i_mesh in i_meshes {
        let mut shape_file =
            File::create(output_path.join(format!("mesh_{}.bin", mesh_counter))).unwrap();
        for i_face in i_mesh.faces.clone() {
            shape_file.write_le_to_u16(i_face.0).unwrap();
            shape_file.write_le_to_u16(i_face.1).unwrap();
            shape_file.write_le_to_u16(i_face.2).unwrap();
        }
        mesh_nodes.push(nodes.len());
        nodes.push(Node {
            mesh: Some(mesh_counter),
            children: None,
            translation: None,
            name: None,
            skin: {
                match joints_inv_accessor_id {
                    Some(_) => Some(0),
                    None => None,
                }
            },
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
                        let mut map = BTreeMap::new();
                        map.insert("POSITION".to_owned(), vertices_positions_accessor_id);
                        map.insert("TEXCOORD_0".to_owned(), vertices_uv_maps_accessor_id);
                        map.insert("NORMAL".to_owned(), vertices_normals_accessor_id);
                        // map.insert("TANGENT".to_owned(), vertices_tangents_accessor_id);
                        if let Some(id) = vertices_joints_accessor_id {
                            map.insert("JOINTS_0".to_owned(), id);
                        }
                        if let Some(id) = vertices_weights_accessor_id {
                            map.insert("WEIGHTS_0".to_owned(), id);
                        }
                        map
                    },
                    indices: Some(accessor_id),
                    material: Some(0), // Need to figure out at some point which mesh goes with which texture...
                });
                primitives
            },
        });
        mesh_counter += 1;
    }

    // This node will refer all Mesh nodes as well as the armature root
    let top_node = nodes.len();
    nodes.push(Node {
        mesh: None,
        name: Some("top".to_owned()),
        rotation: None,
        translation: None,
        scale: None,
        skin: None,
        children: Some({
            // Add the armature root as a children (fixes some GlTF viewers)
            let mut ch = Vec::new();
            match root_node_id {
                Some(id) => ch.push(id),
                None => {}
            }
            ch.append(&mut mesh_nodes);
            ch
        }),
    });

    // Push a single node for the entire scene (fixes some GlTF viewers)
    scenes.push(Scene {
        nodes: Some(vec![top_node]),
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
        skins: Some(skins),
        materials: Some(materials),
        textures: Some(textures),
        images: Some(images),
    };
    let mut gltf_out =
        File::create(output_path.join("model.gltf")).expect("Impossible to create file");
    gltf.write_gltf_pretty(&mut gltf_out).unwrap();
}
