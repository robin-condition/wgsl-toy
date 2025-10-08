use std::{path::PathBuf, sync::mpsc::Receiver};
use shaderwheels_logic::rendering::legacy_graphics::process_future;

use shaderwheels_logic::rendering::{self};

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct ShaderFileLocation {
    pub path: PathBuf,
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct ShaderDBEntry {
    pub id: u64,
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub enum ShaderStorageTypePreference {
    File,
    DB,
}

impl ShaderStorageTypePreference {
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn realize(&self) -> Option<ShaderStorageLocation> {
        match self {
            ShaderStorageTypePreference::File => {
                let picker = rfd::AsyncFileDialog::new()
                    .add_filter("ShaderWheels", &["shwl"])
                    .save_file()
                    .await;

                let file_handle = picker?;

                Some(ShaderStorageLocation::File(ShaderFileLocation {
                    path: file_handle.path().to_path_buf(),
                }))
            }
            ShaderStorageTypePreference::DB => {
                todo!()
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn realize(&self) -> Option<ShaderStorageLocation> {
        todo!()
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub enum ShaderStorageLocation {
    File(ShaderFileLocation),
    RemoteDB(ShaderDBEntry),
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub enum ConcreteOrUndecidedLocation {
    Concrete(ShaderStorageLocation),
    Undecided(ShaderStorageTypePreference),
}

impl ConcreteOrUndecidedLocation {
    pub async fn to_real_loc(self) -> Option<ShaderStorageLocation> {
        match self {
            ConcreteOrUndecidedLocation::Undecided(shader_storage_type_preference) => {
                shader_storage_type_preference.realize().await
            }
            ConcreteOrUndecidedLocation::Concrete(loc) => Some(loc),
        }
    }
}

impl Default for ConcreteOrUndecidedLocation {
    #[cfg(target_arch = "wasm32")]
    fn default() -> Self {
        Self::Undecided(ShaderStorageTypePreference::DB)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn default() -> Self {
        Self::Undecided(ShaderStorageTypePreference::File)
    }
}

#[derive(PartialEq, Eq, Clone, serde::Deserialize, serde::Serialize)]
pub struct ShaderInfo {
    pub contents: String,
    pub name: String,
}

impl ShaderInfo {
    async fn save_in_location(&self, loc: &ShaderStorageLocation) -> bool {
        match loc {
            ShaderStorageLocation::File(shader_file_location) => {
                let res = std::fs::write(shader_file_location.path.clone(), self.contents.clone());
                res.is_ok()
            }
            ShaderStorageLocation::RemoteDB(_shader_dbentry) => todo!(),
        }
    }
}

impl Default for ShaderInfo {
    fn default() -> Self {
        Self {
            contents: rendering::DEFAULT_WGSL_COMPUTE.to_string(),
            name: "Untitled Shader".to_string(),
        }
    }
}

#[derive(Default, Clone, serde::Deserialize, serde::Serialize)]
pub struct ShaderStorageConnection {
    location: ConcreteOrUndecidedLocation,

    info_in_location: ShaderInfo,
    pub currently_saving: bool,
}

impl ShaderStorageConnection {
    pub fn get_location(&self) -> &ConcreteOrUndecidedLocation {
        &self.location
    }
    pub fn get_content(&self) -> &ShaderInfo {
        &self.info_in_location
    }
    pub fn new(location: ConcreteOrUndecidedLocation, content: ShaderInfo) -> Self {
        Self {
            location,
            info_in_location: content,
            currently_saving: false,
        }
    }

    pub fn saving_needed(&self, content: &ShaderInfo) -> bool {
        self.info_in_location != *content
    }

    pub fn eligible_to_save(&self, content: &ShaderInfo) -> bool {
        !self.currently_saving && self.saving_needed(content)
    }

    pub async fn save_to_new_info_if_eligible(
        self,
        new_content: ShaderInfo,
    ) -> Option<ShaderStorageConnection> {
        if !self.eligible_to_save(&new_content) {
            return None;
        }

        let concrete_loc = self.location.to_real_loc().await?;

        let success = new_content.save_in_location(&concrete_loc).await;
        if !success {
            return None;
        }

        Some(ShaderStorageConnection {
            location: ConcreteOrUndecidedLocation::Concrete(concrete_loc),
            info_in_location: new_content,
            currently_saving: false,
        })
    }
}

#[derive(Default)]
pub struct ShaderStorageConnectionManager {
    pub connection: ShaderStorageConnection,
    pub channel: Option<Receiver<Option<ShaderStorageConnection>>>,
}

impl ShaderStorageConnectionManager {
    fn save_process_task(&mut self, new_content: &ShaderInfo) {
        if !self.connection.eligible_to_save(new_content) {
            return;
        }

        let connection_copy = self.connection.clone();
        let content = new_content.clone();
        self.connection.currently_saving = true;

        let (writer, reader) = std::sync::mpsc::channel::<Option<ShaderStorageConnection>>();

        let fut = async move {
            let con = connection_copy;
            let res = con.save_to_new_info_if_eligible(content).await;
            let _ = writer.send(res);
        };

        process_future(fut);

        self.channel = Some(reader);
    }

    pub fn start_save(&mut self, new_content: &ShaderInfo) {
        self.save_process_task(new_content);
    }

    pub fn update(&mut self) {
        if let Some(reader) = self.channel.as_ref() {
            if let Ok(res) = reader.try_recv() {
                self.connection.currently_saving = false;
                if let Some(new_connection) = res {
                    self.connection = new_connection;
                }

                self.channel = None;
            }
        }
    }
}
