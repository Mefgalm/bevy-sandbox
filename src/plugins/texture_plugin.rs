use std::collections::HashMap;

use bevy::{
    asset::{HandleId, LoadState},
    prelude::*,
};

use crate::app_state::AppState;

#[derive(Debug, Clone)]
pub struct ImageDescription {}

#[derive(Debug, Clone)]
pub struct AtlasDescription {
    pub width: f32,
    pub height: f32,
    pub cols: usize,
    pub rows: usize,
    pub atlas_handle: Option<Handle<TextureAtlas>>,
}

#[derive(Debug, Clone)]

pub enum TextureType {
    Image(ImageDescription),
    Atlas(AtlasDescription),
}

#[derive(Debug, Clone)]
pub struct TextureDescription {
    texture_type: TextureType,
    path: String,
    handle: Option<Handle<Image>>,
}
impl TextureDescription {
    pub fn new_image(path: String) -> TextureDescription {
        TextureDescription {
            path,
            handle: None,
            texture_type: TextureType::Image(ImageDescription {}),
        }
    }

    pub fn new_atlas(
        path: String,
        width: f32,
        height: f32,
        cols: usize,
        rows: usize,
    ) -> TextureDescription {
        TextureDescription {
            path,
            handle: None,
            texture_type: TextureType::Atlas(AtlasDescription {
                width,
                height,
                cols,
                rows,
                atlas_handle: None,
            }),
        }
    }
}

//Resource
#[derive(Debug)]
pub struct TextureStore {
    texture_descriptions: HashMap<String, TextureDescription>,
}

impl TextureStore {
    pub fn get_image_handle(&self, path: &str) -> Handle<Image> {
        self.texture_descriptions
            .get(path)
            .and_then(|td| td.handle.clone())
            .expect("Handle not found")
    }

    pub fn get_atlas_handle(&self, path: &str) -> Handle<TextureAtlas> {
        self.texture_descriptions
            .get(path)
            .and_then(|td| match &td.texture_type {
                TextureType::Atlas(atlas) => atlas.atlas_handle.clone(),
                _ => None,
            })
            .expect("Handle not found")
    }
}

fn load_textures(mut texture_store: ResMut<TextureStore>, asset_server: Res<AssetServer>) {
    for (_, td) in texture_store.texture_descriptions.iter_mut() {
        td.handle = Some(asset_server.load(&td.path.clone()));
    }
}

fn check_textures(
    mut state: ResMut<State<AppState>>, //TODO add steps to state Setup(usize)
    mut texture_store: ResMut<TextureStore>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let handles = texture_store
        .texture_descriptions
        .iter()
        .map(|(_, td)| td.handle.as_ref().unwrap().id);

    if let LoadState::Loaded = asset_server.get_group_load_state(handles) {
        for (_, td) in texture_store.texture_descriptions.iter_mut() {
            if let TextureType::Atlas(ref mut atlas) = td.texture_type {
                let texture_atlas = TextureAtlas::from_grid(
                    td.handle.as_ref().unwrap().clone(),
                    Vec2::new(atlas.width, atlas.height),
                    atlas.cols,
                    atlas.rows,
                );
                atlas.atlas_handle = Some(texture_atlases.add(texture_atlas));
            }
        }
        state.set(AppState::Active).unwrap();
    }
}

pub struct TextureResourcePlugin(HashMap<String, TextureDescription>);

impl TextureResourcePlugin {
    pub fn new(texture_descriptions: Vec<TextureDescription>) -> TextureResourcePlugin {
        let mut td_map = HashMap::new();
        for td in texture_descriptions {
            td_map.insert(td.path.clone(), td);
        }
        TextureResourcePlugin(td_map)
    }
}
impl Plugin for TextureResourcePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TextureStore {
            texture_descriptions: self.0.clone(),
        })
        .add_system_set(SystemSet::on_enter(AppState::Setup).with_system(load_textures))
        .add_system_set(SystemSet::on_update(AppState::Setup).with_system(check_textures));
    }
}
