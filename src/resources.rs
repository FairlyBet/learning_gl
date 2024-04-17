use crate::{
    data_3d::{self, Mesh, MeshData, Vertex},
    gl_wrappers::{Gl, Texture},
    material::Material,
    scene::Scene,
    scripting::CompiledScript,
    serializable::{self, PBRTextures, ScriptObject},
};
use fxhash::FxHashMap;
use gl::types::GLenum;
use russimp::{
    material::{DataContent, MaterialProperty, PropertyTypeInfo, TextureType},
    scene::{PostProcess, PostProcessSteps},
    Vector2D,
};
use stb_image::image::{self, Image};
use std::{
    ffi::{c_void, CStr, CString},
    fs,
    marker::PhantomData,
    ops::Range,
    path::{self, Path, PathBuf},
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

impl Resource for MeshData {
    fn folder_name() -> &'static Path {
        Path::new("meshes")
    }

    fn acceptable_extensions() -> Vec<String> {
        vec!["fbx".to_string()]
    }
}

impl Resource for CompiledScript {
    fn folder_name() -> &'static Path {
        Path::new("scripts")
    }

    fn acceptable_extensions() -> Vec<String> {
        vec!["lua".to_string()]
    }
}

impl Resource for Scene {
    fn folder_name() -> &'static Path {
        Path::new("scenes")
    }

    fn acceptable_extensions() -> Vec<String> {
        vec!["json".to_string()]
    }
}

pub const DEFAULT_POSTPROCESS: [PostProcess; 5] = [
    PostProcess::Triangulate,
    PostProcess::OptimizeMeshes,
    PostProcess::OptimizeGraph,
    PostProcess::JoinIdenticalVertices,
    PostProcess::ImproveCacheLocality,
];

pub type RangedIndex = Range<usize>;

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

    pub fn contains_resource(&self, name: &str) -> bool {
        self.table.contains_key(name)
    }

    pub fn unload_all(&mut self) {
        self.table.clear();
        self.vec.clear();
    }
}

pub type RangeIndexContainer<Resource> = ResourceContainer<Resource, RangedIndex>;

impl<Resource> RangeIndexContainer<Resource> {
    pub fn push(&mut self, name: &str, mut resource: Vec<Resource>) -> RangedIndex {
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

    pub fn get_resource(&self, idx: &RangedIndex) -> &[Resource] {
        &self.vec[idx.start..idx.end]
    }
}

pub type SingleIndexContainer<Resource> = ResourceContainer<Resource, usize>;

impl<Resource> SingleIndexContainer<Resource> {
    pub fn push_resource(&mut self, name: &str, resource: Resource) -> usize {
        assert!(
            !self.table.contains_key(name),
            "Container already has this resource"
        );
        let idx = self.vec.len();
        _ = self.table.insert(name.to_string(), idx);
        self.vec.push(resource);
        idx
    }

    pub fn get_resource(&self, idx: usize) -> &Resource {
        &self.vec[idx]
    }
}

// Split logic into separate modules such as material manager or mesh manager
#[rustfmt::skip]
pub struct ResourceManager<'a> {
    pd:                 PhantomData<&'a ()>,
    meshes:             RangeIndexContainer<MeshData>,
    textures:           SingleIndexContainer<Texture>,
    materials:          RangeIndexContainer<Material>,
    scripts:            FxHashMap<String, CompiledScript>,
    scenes:             Vec<Scene>,
                        // base_color, metalness, roughness, ao, normal, displacement
}

impl<'a> ResourceManager<'a> {
    pub fn new(_: &'a Gl) -> Self {
        let default_textures = Self::default_textures();
        let mut tex_container = SingleIndexContainer::new();
        tex_container.push_resource("default_base_color", default_textures.0);
        tex_container.push_resource("default_metalness", default_textures.1);
        tex_container.push_resource("default_roughness", default_textures.2);
        tex_container.push_resource("default_ao", default_textures.3);
        tex_container.push_resource("default_normals", default_textures.4);
        tex_container.push_resource("default_displacement", default_textures.5);

        Self {
            pd: PhantomData::default(),
            meshes: RangeIndexContainer::new(),
            textures: SingleIndexContainer::new(),
            materials: RangeIndexContainer::new(),
            scripts: Default::default(),
            scenes: get_paths::<Scene>()
                .iter()
                .map(|path| Scene::new(path))
                .collect(),
        }
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

    pub fn scenes(&self) -> &[Scene] {
        &self.scenes
    }

    pub fn mesh_data(&self) -> &RangeIndexContainer<MeshData> {
        &self.meshes
    }

    pub fn mesh_data_mut(&mut self) -> &mut RangeIndexContainer<MeshData> {
        &mut self.meshes
    }

    pub fn get_mesh_lazily(&mut self, mesh: &serializable::Mesh) -> data_3d::Mesh {
        if !self.meshes.contains_resource(&mesh.path) {
            self.load_mesh(&mesh, DEFAULT_POSTPROCESS.into());
        }
        let mesh_index = self.meshes.get_index(&mesh.path);
        todo!()
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

        let material_index = self.get_material_lazily(&mesh.material, &mesh.path, &scene, &material_indecies);
    }

    fn get_material_lazily(
        &mut self,
        material: &serializable::Material,
        scene_path: &String,
        scene: &russimp::scene::Scene,
        material_indecies: &Vec<u32>,
    ) -> RangedIndex {

        todo!()
    }

    fn load_materials(
        &mut self,
        material: &serializable::Material,
        scene_path: &String,
        scene: &russimp::scene::Scene,
        material_indecies: &Vec<u32>,
    ) {
        match &material.textures {
            serializable::Textures::Own => {
                for index in material_indecies {
                    // TODO: embeded textures loading

                    let m = &scene.materials[*index as usize];

                    // Find properties that are texture file paths
                    let mut tex_files = Vec::<&MaterialProperty>::new();
                    for property in &m.properties {
                        if property.key == "$tex.file" {
                            tex_files.push(property);
                        }
                    }

                    // Filter out only PBR textures
                    let base_color = tex_files
                        .iter()
                        .find(|p| p.semantic == TextureType::BaseColor)
                        .or_else(|| {
                            tex_files
                                .iter()
                                .find(|p| p.semantic == TextureType::Diffuse)
                        })
                        .map(|prop| Self::get_material_texture_path(prop, scene_path))
                        .unwrap_or("default_base_color".to_string());
                    let metalness = tex_files
                        .iter()
                        .find(|p| p.semantic == TextureType::Metalness)
                        .map(|prop| Self::get_material_texture_path(prop, scene_path))
                        .unwrap_or("default_metalness".to_string());
                    let roughness = tex_files
                        .iter()
                        .find(|p| p.semantic == TextureType::Roughness)
                        .map(|prop| Self::get_material_texture_path(prop, scene_path))
                        .unwrap_or("default_roughness".to_string());
                    let ao = tex_files
                        .iter()
                        .find(|p| p.semantic == TextureType::AmbientOcclusion)
                        .map(|prop| Self::get_material_texture_path(prop, scene_path))
                        .unwrap_or("default_ao".to_string());
                    let normals = tex_files
                        .iter()
                        .find(|p| p.semantic == TextureType::Normals)
                        .map(|prop| Self::get_material_texture_path(prop, scene_path))
                        .unwrap_or("default_normals".to_string());
                    let displacement = tex_files
                        .iter()
                        .find(|p| p.semantic == TextureType::Displacement)
                        .map(|prop| Self::get_material_texture_path(prop, scene_path))
                        .unwrap_or("default_displacement".to_string());
                }
            }
            serializable::Textures::Custom {
                base_color,
                metalness,
                roughness,
                ao,
                normals,
                displacement,
            } => {
                // let
            }
        }

        // only for texture files
        for index in material_indecies {
            let (
                base_color_path,
                metalness_path,
                roughness_path,
                ao_path,
                normals_path,
                displacement_path,
            ) = match &material.textures {
                serializable::Textures::Own => {
                    // TODO: embeded textures loading

                    let m = &scene.materials[*index as usize];

                    // Find properties that are texture file paths
                    let mut tex_files = Vec::<&MaterialProperty>::new();
                    for property in &m.properties {
                        if property.key == "$tex.file" {
                            tex_files.push(property);
                        }
                    }

                    // Filter out only pbr textures
                    let base_color = tex_files
                        .iter()
                        .find(|p| p.semantic == TextureType::BaseColor)
                        .or_else(|| {
                            tex_files
                                .iter()
                                .find(|p| p.semantic == TextureType::Diffuse)
                        })
                        .map(|prop| Self::get_material_texture_path(prop, scene_path));
                    let metalness = tex_files
                        .iter()
                        .find(|p| p.semantic == TextureType::Metalness)
                        .map(|prop| Self::get_material_texture_path(prop, scene_path));
                    let roughness = tex_files
                        .iter()
                        .find(|p| p.semantic == TextureType::Roughness)
                        .map(|prop| Self::get_material_texture_path(prop, scene_path));
                    let ao = tex_files
                        .iter()
                        .find(|p| p.semantic == TextureType::AmbientOcclusion)
                        .map(|prop| Self::get_material_texture_path(prop, scene_path));
                    let normal = tex_files
                        .iter()
                        .find(|p| p.semantic == TextureType::Normals)
                        .map(|prop| Self::get_material_texture_path(prop, scene_path));
                    let displacement = tex_files
                        .iter()
                        .find(|p| p.semantic == TextureType::Displacement)
                        .map(|prop| Self::get_material_texture_path(prop, scene_path));

                    (base_color, metalness, roughness, ao, normal, displacement)
                }
                serializable::Textures::Custom {
                    base_color,
                    metalness,
                    roughness,
                    ao,
                    normals,
                    displacement,
                } => (
                    base_color.clone(),
                    metalness.clone(),
                    roughness.clone(),
                    ao.clone(),
                    normals.clone(),
                    displacement.clone(),
                ),
            };

            let mut base_color = self.textures.get_index("default_base_color");
            if let Some(path) = &base_color_path {
                base_color = self.get_tex_lazily(&Self::load_img(&path), &path);
            }

            let mut metalness = self.textures.get_index("default_metalness");
            let mut roughness = self.textures.get_index("default_roughness");
            let mut ao = self.textures.get_index("default_ao");

            match &material.pbr_channels {
                PBRTextures::Separated => {
                    if let Some(path) = &metalness_path {
                        metalness = self.get_tex_lazily(&Self::load_img(&path), &path);
                    }
                    if let Some(path) = &roughness_path {
                        roughness = self.get_tex_lazily(&Self::load_img(&path), &path);
                    }
                    if let Some(path) = &ao_path {
                        ao = self.get_tex_lazily(&Self::load_img(&path), &path);
                    }
                }
                PBRTextures::Merged(pbr_channels) => {
                    if let Some(path) = metalness_path
                        .as_ref()
                        .or(roughness_path.as_ref())
                        .or(ao_path.as_ref())
                    {
                        let img = Self::load_img(path);

                        if let Some(path) = &metalness_path {
                            let img = Self::extract_channel(&img, pbr_channels.metalness_offset());
                            let mut p = PathBuf::from(pbr_channels.metalness_offset().to_string());
                            p.push(path);
                            metalness = self.get_tex_lazily(&img, p.to_str().unwrap());
                        }

                        if let Some(path) = &roughness_path {
                            let img = Self::extract_channel(&img, pbr_channels.roughness_offset());
                            let mut p = PathBuf::from(pbr_channels.roughness_offset().to_string());
                            p.push(path);
                            roughness = self.get_tex_lazily(&img, p.to_str().unwrap());
                        }

                        if let Some(path) = &ao_path {
                            let img = Self::extract_channel(&img, pbr_channels.ao_offset());
                            let mut p = PathBuf::from(pbr_channels.ao_offset().to_string());
                            p.push(path);
                            ao = self.get_tex_lazily(&img, p.to_str().unwrap());
                        }
                    }
                }
            }

            let mut normals = self.textures.get_index("default_normals");
            if let Some(path) = &normals_path {
                normals = self.get_tex_lazily(&Self::load_img(&path), &path)
            }

            let mut displacement = self.textures.get_index("default_displacement");
            if let Some(path) = &displacement_path {
                displacement = self.get_tex_lazily(&Self::load_img(&path), &path)
            }

            // let material = Material{
            //     base_color,
            //     metalness,
            //     roughness,
            //     ao,
            //     normals,
            //     displacement,
            // };
        }
    }

    fn get_material_texture_path(prop: &MaterialProperty, scene_path: &str) -> String {
        if let PropertyTypeInfo::String(s) = &prop.data {
            let mut path = PathBuf::from(&scene_path);
            path.pop();
            path.push(PathBuf::from(&s));
            return path.to_str().unwrap().to_string();
        }
        unreachable!()
    }

    fn load_img(path: &str) -> Image<u8> {
        // let filename = CString::new(path.as_bytes()).unwrap();
        // let mut x = 0;
        // let mut y = 0;
        // let mut channels = 0;
        // let data = unsafe {
        //     // stb_image::stbi_set_flip_vertically_on_load(1); // Could be needed
        //     stb_image::stbi_load(filename.as_ptr(), &mut x, &mut y, &mut channels, 0)
        // };
        // assert_ne!(data, std::ptr::null_mut());
        // unsafe { stb_image::stbi_image_free(data.cast()); }

        match image::load(path) {
            image::LoadResult::Error(s) => panic!("{}", s),
            image::LoadResult::ImageU8(img) => img,
            image::LoadResult::ImageF32(_) => unreachable!(),
        }
    }

    fn extract_channel(img: &Image<u8>, channel_offset: usize) -> Image<u8> {
        let channel_count = img.data.len() / img.width / img.height;
        let mut res = Vec::with_capacity(img.width * img.height);
        for y in 0..img.height {
            for x in (0..img.width).step_by(channel_count) {
                res.push(img.data[y * img.width + x + channel_offset]);
            }
        }

        Image::new(img.width, img.height, img.depth, res)
    }

    fn get_tex_lazily(&mut self, img: &Image<u8>, path: &str) -> usize {
        if self.textures.contains_resource(path) {
            return self.textures.get_index(path);
        }

        let channel_count = img.data.len() / img.width / img.height;
        let format = if channel_count == 1 {
            gl::RED
        } else if channel_count == 3 {
            gl::RGB // or SRGB
        } else if channel_count == 4 {
            gl::RGBA
        } else {
            unreachable!()
        };

        let tex = Texture::new(gl::TEXTURE_2D).unwrap();
        tex.bind();
        tex.texture_data(
            (img.width as i32, img.height as i32),
            img.data.as_ptr().cast(),
            gl::UNSIGNED_BYTE,
            format,
            format,
        );
        tex.generate_mipmaps();
        tex.parameter(gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE); // Not sure about CLAMP_TO_EDGE, could be REPEAT
        tex.parameter(gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE);
        tex.parameter(gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR);
        tex.parameter(gl::TEXTURE_MAG_FILTER, gl::LINEAR);

        self.textures.push_resource(path, tex)
    }

    pub fn get_script(&self, script: &ScriptObject) -> String {
        // will be replaced later with some binary storing logic
        fs::read_to_string(&script.script_path).unwrap()
    }
}
