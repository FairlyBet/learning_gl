use crate::{
    data3d::{Mesh, MeshData, Vertex},
    gl_wrappers::{Gl, Texture},
    material::Material,
    scene::Scene,
    scripting::CompiledScript,
    serializable::{self, PBRTextures, ScriptObject},
    utils::StbImage,
};
use fxhash::FxHashMap;
use gl::types::GLenum;
use russimp::{
    material::{MaterialProperty, PropertyTypeInfo, TextureType},
    scene::{PostProcess, PostProcessSteps},
    Vector2D,
};
use std::{
    fs,
    marker::PhantomData,
    ops::Range,
    path::{Path, PathBuf},
    ptr,
    str::FromStr as _,
};

pub fn get_paths<T>() -> Vec<String>
where
    T: Resource,
{
    const ASSETS_DIR: &str = "assets";
    fs::create_dir_all(ASSETS_DIR).unwrap();
    let mut path = PathBuf::from_str(ASSETS_DIR).unwrap();
    path.push(T::folder_name());

    fn search<T>(path: &Path) -> Vec<String>
    where
        T: Resource,
    {
        let mut result = vec![];
        let entries = fs::read_dir(path).unwrap();

        for entry in entries {
            let entry = entry.unwrap();

            if let Some(extension) = entry.path().extension() {
                if T::acceptable_extensions().contains(&extension.to_str().unwrap().to_string()) {
                    result.push(entry.path().into_os_string().into_string().unwrap());
                    continue;
                }
            }
            if entry.file_type().unwrap().is_dir() {
                result.append(&mut search::<T>(&entry.path()));
            }
        }
        result
    }

    search::<T>(&path)
}

pub trait Resource {
    fn folder_name() -> &'static Path;
    fn acceptable_extensions() -> Vec<String>;
}

impl Resource for Scene {
    fn folder_name() -> &'static Path {
        Path::new("scenes")
    }

    fn acceptable_extensions() -> Vec<String> {
        vec!["json".to_string()]
    }
}

pub type RangeIndex = Range<usize>;

pub struct ResourceContainer<Resource, ResourceIndex: Clone> {
    // Add resource releasing logic along with free indecies accounting logic
    table: FxHashMap<String, ResourceIndex>,
    vec: Vec<Resource>,
}

impl<Resource, ResourceIndex: Clone> ResourceContainer<Resource, ResourceIndex> {
    pub fn new() -> Self {
        Self {
            table: Default::default(),
            vec: Vec::new(),
        }
    }

    pub fn get_index(&self, name: &str) -> ResourceIndex {
        self.table[name].clone()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.table.contains_key(name)
    }

    pub fn unload_all(&mut self) {
        self.table.clear();
        self.vec.clear();
    }
}

pub type RangeIndexContainer<Resource> = ResourceContainer<Resource, RangeIndex>;

impl<Resource> RangeIndexContainer<Resource> {
    pub fn push(&mut self, name: &str, mut resource: Vec<Resource>) -> RangeIndex {
        assert!(
            !self.table.contains_key(name),
            "Container already has this resource"
        );
        let idx = Range {
            start: self.vec.len(),
            end: self.vec.len() + resource.len(),
        };
        _ = self.table.insert(name.to_string(), idx.clone());
        self.vec.append(&mut resource);
        idx
    }

    pub fn get(&self, idx: &RangeIndex) -> &[Resource] {
        &self.vec[idx.start..idx.end]
    }
}

pub type SingleIndexContainer<Resource> = ResourceContainer<Resource, usize>;

impl<Resource> SingleIndexContainer<Resource> {
    pub fn push(&mut self, name: &str, resource: Resource) -> usize {
        assert!(
            !self.table.contains_key(name),
            "Container already has this resource"
        );
        let idx = self.vec.len();
        _ = self.table.insert(name.to_string(), idx);
        self.vec.push(resource);
        idx
    }

    pub fn get(&self, idx: usize) -> &Resource {
        &self.vec[idx]
    }
}

pub struct ResourceManager<'a> {
    pd: PhantomData<&'a ()>,
    mesh_manager: MeshManager,
    scripts: FxHashMap<String, CompiledScript>,
    scenes: Vec<Scene>,
}

impl<'a> ResourceManager<'a> {
    pub fn new(_: &'a Gl) -> Self {
        Self {
            pd: PhantomData::default(),
            mesh_manager: MeshManager::new(),
            scripts: Default::default(),
            scenes: get_paths::<Scene>()
                .iter()
                .map(|path| Scene::new(path))
                .collect(),
        }
    }

    pub fn mesh_manager(&self) -> &MeshManager {
        &self.mesh_manager
    }

    pub fn mesh_manager_mut(&mut self) -> &mut MeshManager {
        &mut self.mesh_manager
    }

    pub fn scenes(&self) -> &[Scene] {
        &self.scenes
    }

    pub fn get_script(&self, script: &ScriptObject) -> String {
        // will be replaced later with some binary storing logic
        fs::read_to_string(&script.script_path).unwrap()
    }
}

pub struct MeshManager {
    meshes: RangeIndexContainer<MeshData>,
    materials: RangeIndexContainer<Material>,
    textures: SingleIndexContainer<Texture>,
}

impl MeshManager {
    const DEFAULT_POSTPROCESS: [PostProcess; 5] = [
        PostProcess::Triangulate,
        PostProcess::OptimizeMeshes,
        PostProcess::OptimizeGraph,
        PostProcess::JoinIdenticalVertices,
        PostProcess::ImproveCacheLocality,
    ];

    pub fn new() -> Self {
        let mut textures = SingleIndexContainer::<Texture>::new();
        let default_textures = Self::default_textures();
        textures.push("default_base_color", default_textures.0);
        textures.push("default_metalness", default_textures.1);
        textures.push("default_roughness", default_textures.2);
        textures.push("default_ao", default_textures.3);
        textures.push("default_normals", default_textures.4);
        textures.push("default_displacement", default_textures.5);

        Self {
            meshes: RangeIndexContainer::new(),
            materials: RangeIndexContainer::new(),
            textures,
        }
    }

    pub fn meshes(&self) -> &RangeIndexContainer<MeshData> {
        &self.meshes
    }

    pub fn materials(&self) -> &RangeIndexContainer<Material> {
        &self.materials
    }

    pub fn textures(&self) -> &SingleIndexContainer<Texture> {
        &self.textures
    }

    fn default_textures() -> (Texture, Texture, Texture, Texture, Texture, Texture) {
        let size = (1, 1);

        let data = (128u8, 128u8, 128u8);
        let base_color = Self::create_dafault_texture(ptr::from_ref(&data.0), size, gl::RGB);

        let data = 0u8;
        let metalness = Self::create_dafault_texture(ptr::from_ref(&data), size, gl::RED);

        let data = 128u8;
        let roughness = Self::create_dafault_texture(ptr::from_ref(&data), size, gl::RED);

        let data = 255u8;
        let ao = Self::create_dafault_texture(ptr::from_ref(&data), size, gl::RED);

        let data = (0u8, 0u8, 255u8); // Set proper value
        let normals = Self::create_dafault_texture(ptr::from_ref(&data.0), size, gl::RGB);

        let data = 255u8; // Set proper value
        let displacement = Self::create_dafault_texture(ptr::from_ref(&data), size, gl::RED);

        (base_color, metalness, roughness, ao, normals, displacement)
    }

    fn create_dafault_texture(data: *const u8, size: (i32, i32), format: GLenum) -> Texture {
        let tex = Texture::new(gl::TEXTURE_2D).unwrap();
        tex.bind();
        tex.texture_data(size, data.cast(), gl::UNSIGNED_BYTE, format, format);
        tex.parameter(gl::TEXTURE_WRAP_S, gl::REPEAT);
        tex.parameter(gl::TEXTURE_WRAP_T, gl::REPEAT);
        tex.parameter(gl::TEXTURE_MIN_FILTER, gl::NEAREST);
        tex.parameter(gl::TEXTURE_MAG_FILTER, gl::NEAREST);
        tex
    }

    pub fn get_mesh_lazily(&mut self, mesh: &serializable::Mesh) -> Mesh {
        if !self.meshes.contains(&mesh.path) {
            self.load_mesh(&mesh, Self::DEFAULT_POSTPROCESS.into());
        }
        let mesh_index = self.meshes.get_index(&mesh.path);
        let material_index = self.materials.get_index(&mesh.path);

        Mesh {
            mesh_index,
            material_index,
        }
    }

    fn load_mesh(&mut self, mesh: &serializable::Mesh, post_process: PostProcessSteps) {
        let scene = russimp::scene::Scene::from_file(&mesh.path, post_process).unwrap();

        let mut submeshes_data = Vec::with_capacity(scene.meshes.len());
        let mut material_indecies = Vec::with_capacity(scene.meshes.len());
        let mut vertex_data = Vec::<Vertex>::new();
        let mut index_data = Vec::<u32>::new();

        for submesh in &scene.meshes {
            // processing submeshes
            for i in 0..submesh.vertices.len() {
                let position = submesh.vertices[i];
                let normal = submesh.normals[i];
                let tex_coord =
                    submesh.texture_coords[0]
                        .as_ref()
                        .map_or(Vector2D::default(), |coords| Vector2D {
                            x: coords[i].x,
                            y: coords[i].y,
                        });

                let vertex = Vertex {
                    position,
                    normal,
                    tex_coord,
                };
                vertex_data.push(vertex);
            }

            for face in &submesh.faces {
                for index in &face.0 {
                    index_data.push(*index);
                }
            }

            let submesh_data =
                MeshData::from_vertex_index_data(&vertex_data, &index_data, gl::STATIC_DRAW);
            submeshes_data.push(submesh_data);
            material_indecies.push(submesh.material_index);

            vertex_data.clear();
            index_data.clear();
        }

        _ = self.meshes.push(&mesh.path, submeshes_data);

        self.load_material(&scene, &material_indecies, &mesh.path, &mesh.material_info);

        // self.get_material_lazily(material, scene_path, &scene, &material_indecies);

        // _ = self.get_material_lazily(&mesh.material, &mesh.path, &scene, &material_indecies);
    }

    fn load_material(
        &mut self,
        scene: &russimp::scene::Scene,
        material_indecies: &Vec<u32>,
        mesh_path: &str,
        material_info: &serializable::MaterialInfo,
    ) {
        let mut material_items = Vec::with_capacity(material_indecies.len());
        let mut tex_files = Vec::new();

        for index in material_indecies {
            // TODO: embeded textures loading

            let m = &scene.materials[*index as usize];

            tex_files.clear();

            // Find properties that are texture file paths
            m.properties
                .iter()
                .filter(|item| item.key == "$tex.file")
                .for_each(|item| tex_files.push(item));

            // for property in &m.properties {
            //     if property.key == "$tex.file" {
            //         // "$tex.file" is just how it is marked by Assimp
            //         tex_files.push(property);
            //     }
            // }

            // Filter out only PBR textures
            let base_color = tex_files
                .iter()
                .find(|p| p.semantic == TextureType::BaseColor)
                .or_else(|| {
                    tex_files
                        .iter()
                        .find(|p| p.semantic == TextureType::Diffuse)
                })
                .map(|prop| Self::get_texture_path(prop, mesh_path));
            let metalness = tex_files
                .iter()
                .find(|p| p.semantic == TextureType::Metalness)
                .map(|prop| Self::get_texture_path(prop, mesh_path));
            let roughness = tex_files
                .iter()
                .find(|p| p.semantic == TextureType::Roughness)
                .map(|prop| Self::get_texture_path(prop, mesh_path));
            let ao = tex_files
                .iter()
                .find(|p| p.semantic == TextureType::AmbientOcclusion)
                .map(|prop| Self::get_texture_path(prop, mesh_path));
            let normals = tex_files
                .iter()
                .find(|p| p.semantic == TextureType::Normals)
                .map(|prop| Self::get_texture_path(prop, mesh_path));
            let displacement = tex_files
                .iter()
                .find(|p| p.semantic == TextureType::Displacement)
                .map(|prop| Self::get_texture_path(prop, mesh_path));

            material_items.push((base_color, metalness, roughness, ao, normals, displacement));
        }

        let mut items = Vec::with_capacity(material_items.len());
        for item in material_items {
            items.push(self.load_material_textures(
                material_info,
                &(item.0, item.1, item.2, item.3, item.4, item.5),
            ));
        }

        _ = self.materials.push(mesh_path, items);

        // let items = match &material.textures {
        //     serializable::Textures::Own => {
        //         items
        //     }
        //     serializable::Textures::Custom {
        //         base_color,
        //         metalness,
        //         roughness,
        //         ao,
        //         normals,
        //         displacement,
        //     } => {
        //         let m = self.load_material_textures(
        //             material,
        //             (base_color, metalness, roughness, ao, normals, displacement),
        //         );
        //         vec![m]
        //     }
        // };
    }

    fn get_texture_path(prop: &MaterialProperty, mesh_path: &str) -> String {
        if let PropertyTypeInfo::String(s) = &prop.data {
            let mut path = PathBuf::from(&mesh_path);
            path.pop();
            path.push(PathBuf::from(&s));
            return path.to_str().unwrap().to_string();
        }
        unreachable!()
    }

    fn load_material_textures(
        &mut self,
        material: &serializable::MaterialInfo,
        texs: &(
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
        ),
    ) -> Material {
        let mut base_color = self.textures.get_index("default_base_color");
        if let Some(path) = &texs.0 {
            let img = StbImage::load(path, true);
            base_color = self.load_tex(img.data(), (img.x(), img.y()), img.channels(), path, true);
        }

        let mut metalness = self.textures.get_index("default_metalness");
        let mut roughness = self.textures.get_index("default_roughness");
        let mut ao = self.textures.get_index("default_ao");

        match &material.pbr_channels {
            PBRTextures::Separated => {
                if let Some(path) = &texs.1 {
                    let img = StbImage::load(path, true);
                    metalness =
                        self.load_tex(img.data(), (img.x(), img.y()), img.channels(), path, false);
                }
                if let Some(path) = &texs.2 {
                    let img = StbImage::load(path, true);
                    roughness =
                        self.load_tex(img.data(), (img.x(), img.y()), img.channels(), path, false);
                }
                if let Some(path) = &texs.3 {
                    let img = StbImage::load(path, true);
                    ao = self.load_tex(img.data(), (img.x(), img.y()), img.channels(), path, false);
                }
            }
            PBRTextures::Merged(pbr_channels) => {
                if let Some(path) = texs.1.as_ref().or(texs.2.as_ref()).or(texs.3.as_ref()) {
                    let img = StbImage::load(path, true);

                    if let Some(path) = &texs.1 {
                        let ch = img.extract_channel(pbr_channels.metalness_offset());
                        let mut p = PathBuf::from(pbr_channels.metalness_offset().to_string());
                        p.push(path);
                        metalness =
                            self.load_tex(&ch, (img.x(), img.y()), 1, p.to_str().unwrap(), false);
                    }

                    if let Some(path) = &texs.2 {
                        let ch = img.extract_channel(pbr_channels.roughness_offset());
                        let mut p = PathBuf::from(pbr_channels.roughness_offset().to_string());
                        p.push(path);
                        roughness =
                            self.load_tex(&ch, (img.x(), img.y()), 1, p.to_str().unwrap(), false);
                    }

                    if let Some(path) = &texs.3 {
                        let ch = img.extract_channel(pbr_channels.ao_offset());
                        let mut p = PathBuf::from(pbr_channels.ao_offset().to_string());
                        p.push(path);
                        ao = self.load_tex(&ch, (img.x(), img.y()), 1, p.to_str().unwrap(), false);
                    }
                }
            }
        }

        let mut normals = self.textures.get_index("default_normals");
        if let Some(path) = &texs.4 {
            let img = StbImage::load(path, true);
            normals = self.load_tex(img.data(), (img.x(), img.y()), img.channels(), path, false);
        }

        let mut displacement = self.textures.get_index("default_displacement");
        if let Some(path) = &texs.5 {
            let img = StbImage::load(path, true);
            displacement =
                self.load_tex(img.data(), (img.x(), img.y()), img.channels(), path, false);
        }

        Material {
            base_color,
            metalness,
            roughness,
            ao,
            normals,
            displacement,
        }
    }

    fn load_tex(
        &mut self,
        img: &[u8],
        size: (usize, usize),
        channels: usize,
        path: &str,
        srgb: bool,
    ) -> usize {
        if self.textures.contains(path) {
            return self.textures.get_index(path);
        }

        let (format, internal_format) = if channels == 1 {
            (gl::RED, gl::RED)
        } else if channels == 3 {
            (gl::RGB, gl::RGB * (!srgb as u32) + gl::SRGB * (srgb as u32)) // or SRGB
        } else if channels == 4 {
            (
                gl::RGBA,
                gl::RGBA * (!srgb as u32) + gl::SRGB_ALPHA * (srgb as u32),
            )
        } else {
            unreachable!()
        };

        let tex = Texture::new(gl::TEXTURE_2D).unwrap();
        tex.bind();
        tex.texture_data(
            (size.0 as i32, size.1 as i32),
            img.as_ptr().cast(),
            gl::UNSIGNED_BYTE,
            format,
            internal_format,
        );
        tex.generate_mipmaps();
        tex.parameter(gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE); // Not sure about CLAMP_TO_EDGE, could be REPEAT
        tex.parameter(gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE);
        tex.parameter(gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR);
        tex.parameter(gl::TEXTURE_MAG_FILTER, gl::LINEAR);

        self.textures.push(path, tex)
    }

    // fn get_material_lazily(
    //     &mut self,
    //     material: &serializable::MaterialInfo,
    //     scene_path: &str,
    //     scene: &russimp::scene::Scene,
    //     material_indecies: &Vec<u32>,
    // ) -> RangeIndex {
    //     let material_path = Self::material_name(material, scene_path);
    //     if !self.materials.contains(&material_path) {
    //         self.load_material(material, &material_path, scene, material_indecies);
    //     }
    //     self.materials.get_index(&material_path)
    // }

    // fn material_name(material: &serializable::MaterialInfo, scene_path: &str) -> String {
    //     match &material.textures {
    //         serializable::Textures::Own => scene_path.to_string(),
    //         serializable::Textures::Custom {
    //             base_color,
    //             metalness,
    //             roughness,
    //             ao,
    //             normals,
    //             displacement,
    //         } => Self::custom_material_name(
    //             base_color,
    //             metalness,
    //             roughness,
    //             ao,
    //             normals,
    //             displacement,
    //         ),
    //     }
    // }

    // fn custom_material_name(
    //     base_color: &Option<String>,
    //     metalness: &Option<String>,
    //     roughness: &Option<String>,
    //     ao: &Option<String>,
    //     normals: &Option<String>,
    //     displacement: &Option<String>,
    // ) -> String {
    //     let base_color = base_color
    //         .clone()
    //         .unwrap_or("default_base_color".to_string());
    //     let metalness = metalness.clone().unwrap_or("default_metalness".to_string());
    //     let roughness = roughness.clone().unwrap_or("default_roughness".to_string());
    //     let ao = ao.clone().unwrap_or("default_ao".to_string());
    //     let normals = normals.clone().unwrap_or("default_normals".to_string());
    //     let displacement = displacement
    //         .clone()
    //         .unwrap_or("default_displacement".to_string());
    //     let mut path = String::with_capacity(
    //         base_color.len()
    //             + metalness.len()
    //             + roughness.len()
    //             + ao.len()
    //             + normals.len()
    //             + displacement.len(),
    //     );
    //     path.push_str(&base_color);
    //     path.push_str(&metalness);
    //     path.push_str(&roughness);
    //     path.push_str(&ao);
    //     path.push_str(&normals);
    //     path.push_str(&displacement);
    //     path
    // }
}
