use crate::{
    data_3d::{self, Mesh, MeshData, VertexData},
    scene::Scene,
    scripting::CompiledScript,
    serializable::{self, ScriptObject},
};
use fxhash::FxHashMap;
use russimp::{
    scene::{PostProcess, PostProcessSteps},
    Vector2D,
};
use std::{
    fs,
    ops::Range,
    path::{Path, PathBuf},
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

    fn search<T>(path: &Path) -> Vec<ResourcePath>
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

pub type ResourcePath = String;
pub type RangedIndex = Range<usize>;

pub struct ResourceContainer<Resource, ResourceIndex: Clone> {
    table: FxHashMap<ResourcePath, ResourceIndex>,
    vec: Vec<Resource>,
}

impl<Resource, ResourceIndex: Clone> ResourceContainer<Resource, ResourceIndex> {
    pub fn new() -> Self {
        Self {
            table: Default::default(),
            vec: Vec::new(),
        }
    }

    pub fn get_index(&self, name: &ResourcePath) -> ResourceIndex {
        self.table[name].clone()
    }

    pub fn contains_resource(&self, name: &ResourcePath) -> bool {
        self.table.contains_key(name)
    }

    pub fn unload_all(&mut self) {
        self.table.clear();
        self.vec.clear();
    }
}

pub type RangeContainer<Resource> = ResourceContainer<Resource, RangedIndex>;

impl<Resource> RangeContainer<Resource> {
    pub fn push_resources(
        &mut self,
        name: &ResourcePath,
        mut resources: Vec<Resource>,
    ) -> RangedIndex {
        let idx = Range {
            start: self.vec.len(),
            end: self.vec.len() + resources.len(),
        };
        assert!(
            self.table.insert(name.clone(), idx.clone()).is_none(),
            "Container already has this resource"
        );
        self.vec.append(&mut resources);
        idx
    }

    pub fn soft_push(&mut self, name: &ResourcePath, resources: Vec<Resource>) -> RangedIndex {
        if self.table.contains_key(name) {
            self.table[name].clone()
        } else {
            self.push_resources(name, resources)
        }
    }

    pub fn get_resource(&self, idx: &RangedIndex) -> &[Resource] {
        &self.vec[idx.start..idx.end]
    }
}

pub type IndexContainer<Resource> = ResourceContainer<Resource, usize>;

impl<Resource> IndexContainer<Resource> {
    pub fn push_resource(&mut self, name: &ResourcePath, resource: Resource) -> usize {
        assert!(
            !self.table.contains_key(name),
            "Container already has this resource"
        );
        let idx = self.vec.len();
        _ = self.table.insert(name.clone(), idx);
        self.vec.push(resource);
        idx
    }

    pub fn get_resource(&self, idx: usize) -> &Resource {
        &self.vec[idx]
    }
}

pub struct ResourceManager {
    meshes: RangeContainer<MeshData>,
    scripts: FxHashMap<ResourcePath, CompiledScript>,
    scenes: Vec<Scene>,
}

impl ResourceManager {
    pub fn new() -> Self {
        Self {
            meshes: RangeContainer::new(),
            scripts: Default::default(),
            scenes: get_paths::<Scene>()
                .iter()
                .map(|path| Scene::new(path))
                .collect(),
        }
    }

    pub fn scenes(&self) -> &[Scene] {
        &self.scenes
    }

    pub fn mesh_data(&self) -> &RangeContainer<MeshData> {
        &self.meshes
    }

    pub fn mesh_data_mut(&mut self) -> &mut RangeContainer<MeshData> {
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

        let mut meshes = Vec::with_capacity(scene.meshes.len());
        for submesh in &scene.meshes {
            let vertex_count = submesh.vertices.len();
            let mut vertex_data = Vec::<VertexData>::with_capacity(vertex_count);
            for i in 0..vertex_count {
                let position = submesh.vertices[i];
                let normal = submesh.normals[i];
                let tex_coord =
                    submesh.texture_coords[0]
                        .as_ref()
                        .map_or(Vector2D::default(), |coords| Vector2D {
                            x: coords[i].x,
                            y: coords[i].y,
                        });
                let vertex = VertexData {
                    position,
                    normal,
                    tex_coord,
                };
                vertex_data.push(vertex);
            }

            // 1 face contains 3 indexes
            let mut index_data = Vec::<u32>::with_capacity(submesh.faces.len() * 3);
            for face in &submesh.faces {
                for index in &face.0 {
                    index_data.push(*index);
                }
            }

            let material_index = submesh.material_index as usize;

            for info in mesh.material.iter() {
                match info {
                    serializable::MateialInfo::None => todo!(),
                    serializable::MateialInfo::Default => {}
                    serializable::MateialInfo::Custom(_) => todo!(),
                }
            }

            let mesh = MeshData::from_vertex_index_data(&vertex_data, &index_data, gl::STATIC_DRAW);
            meshes.push(mesh);
        }
        // meshes
    }

    fn load_mateials(&mut self) {}

    pub fn get_script(&self, script: &ScriptObject) -> String {
        // will be replaced later with some binary storing logic
        fs::read_to_string(&script.script_path).unwrap()
    }
}
