use bevy::{prelude::*, sprite::Anchor};

use bevy_picoui::{
    pico::{ItemStyle, Pico, Pico2dCamera, PicoItem},
    PicoPlugin,
};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_plugins(DefaultPlugins)
        .add_plugins(PicoPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), Pico2dCamera));
}

struct NinePatchImages {
    blue_button: ((u32, u32, u32, u32), Handle<Image>, Handle<Image>),
    yellow_button: ((u32, u32, u32, u32), Handle<Image>, Handle<Image>),
    blue_panel: ((u32, u32, u32, u32), Handle<Image>),
    grey_panel: ((u32, u32, u32, u32), Handle<Image>),
}

fn update(
    mut pico: ResMut<Pico>,
    asset_server: ResMut<AssetServer>,
    mut nine_patch_images: Local<Option<NinePatchImages>>,
    mut font: Local<Option<Handle<Font>>>,
) {
    if nine_patch_images.is_none() {
        // For actual projects consider using https://github.com/NiklasEi/bevy_asset_loader or load assets in separate startup system
        *nine_patch_images = Some(NinePatchImages {
            blue_button: (
                // Units are pixels: Left, Top, Right, Bottom
                (7, 7, 7, 28),
                asset_server
                    .load("kenney_ui-pack/PNG/blue_button07.png")
                    .into(),
                asset_server
                    .load("kenney_ui-pack/PNG/blue_button08.png")
                    .into(),
            ),
            yellow_button: (
                (7, 7, 7, 28),
                asset_server
                    .load("kenney_ui-pack/PNG/yellow_button07.png")
                    .into(),
                asset_server
                    .load("kenney_ui-pack/PNG/yellow_button08.png")
                    .into(),
            ),
            blue_panel: (
                (7, 7, 7, 7),
                asset_server
                    .load("kenney_ui-pack/PNG/blue_panel.png")
                    .into(),
            ),
            grey_panel: (
                (7, 7, 7, 7),
                asset_server
                    .load("kenney_ui-pack/PNG/grey_panel.png")
                    .into(),
            ),
        });
        *font = Some(
            asset_server
                .load("kenney_ui-pack/Font/kenvector_future.ttf")
                .into(),
        );
    }
    let nine_patch = nine_patch_images.as_mut().unwrap();
    let font = font.as_mut().unwrap();

    let blue_panel = pico.add(PicoItem {
        y: Val::Percent(50.0),
        x: Val::Percent(50.0),
        width: Val::Vh(70.0),
        height: Val::Vh(50.0),
        anchor: Anchor::Center,
        style: ItemStyle {
            /// For image to be fully opaque with the correct colors, the background needs to be white.
            background_color: Color::WHITE,
            nine_patch: Some(nine_patch.blue_panel.0),
            image: Some(nine_patch.blue_panel.1.clone_weak()),
            ..default()
        },
        ..default()
    });

    pico.add(PicoItem {
        text: String::from("SUPER AWESOME GAME"),
        width: Val::Percent(100.0),
        height: Val::Percent(12.0),
        anchor: Anchor::TopCenter,
        anchor_parent: Anchor::TopCenter,
        parent: Some(blue_panel),
        style: ItemStyle {
            anchor_text: Anchor::Center,
            font_size: Val::Vh(2.5),
            font: font.clone_weak(),
            ..default()
        },
        ..default()
    });

    let grey_panel = pico.add(PicoItem {
        width: Val::Percent(100.0),
        height: Val::Percent(88.0),
        anchor: Anchor::BottomCenter,
        anchor_parent: Anchor::BottomCenter,
        parent: Some(blue_panel),
        style: ItemStyle {
            background_color: Color::WHITE,
            nine_patch: Some(nine_patch.grey_panel.0),
            image: Some(nine_patch.grey_panel.1.clone_weak()),
            ..default()
        },
        ..default()
    });

    let main_panel = pico.add(PicoItem {
        width: Val::Percent(70.0),
        height: Val::Percent(70.0),
        anchor: Anchor::Center,
        anchor_parent: Anchor::Center,
        parent: Some(grey_panel),
        ..default()
    });

    {
        let _guard = pico.vstack(Val::Percent(5.0), Val::Percent(5.0), false, &main_panel);

        let btn_template = PicoItem {
            width: Val::Percent(100.0),
            height: Val::Percent(25.0),
            anchor: Anchor::TopCenter,
            anchor_parent: Anchor::TopCenter,
            parent: Some(main_panel),
            style: ItemStyle {
                font: font.clone_weak(),
                font_size: Val::Vh(2.5),
                background_color: Color::WHITE,
                ..default()
            },
            ..default()
        };

        let mut btn = btn_template.clone();
        btn.text = String::from("START GAME");
        btn.style.text_color = Color::rgb(0.3, 0.3, 0.3);
        btn.height = Val::Percent(32.0);
        btn.style.nine_patch = Some(nine_patch.blue_button.0);
        let btn_idx = pico.add(btn);
        if pico.hovered(&btn_idx) {
            pico.get_mut(&btn_idx).style.image = Some(nine_patch.yellow_button.2.clone_weak());
        } else {
            pico.get_mut(&btn_idx).style.image = Some(nine_patch.yellow_button.1.clone_weak());
        }

        let mut btn = btn_template.clone();
        btn.text = String::from("OPTIONS");
        btn.style.nine_patch = Some(nine_patch.blue_button.0);
        let btn_idx = pico.add(btn);
        if pico.hovered(&btn_idx) {
            pico.get_mut(&btn_idx).style.image = Some(nine_patch.blue_button.2.clone_weak());
        } else {
            pico.get_mut(&btn_idx).style.image = Some(nine_patch.blue_button.1.clone_weak());
        }

        let mut btn = btn_template.clone();
        btn.text = String::from("CREDITS");
        btn.style.nine_patch = Some(nine_patch.blue_button.0);
        let btn_idx = pico.add(btn);
        if pico.hovered(&btn_idx) {
            pico.get_mut(&btn_idx).style.image = Some(nine_patch.blue_button.2.clone_weak());
        } else {
            pico.get_mut(&btn_idx).style.image = Some(nine_patch.blue_button.1.clone_weak());
        }
    }
}
