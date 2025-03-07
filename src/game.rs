//= IMPORTS =======================================================================================

use crate::player::Player;
use crate::world::{NodeSeq, World, WorldGen};

use voxel_math::dda::HitResult;
use voxel_render::{Material, Renderer, Settings, Voxel, WorldData, VOXEL_MATERIALS};
use voxel_winput::{mapping::InputKind, window::Window};

use glam::{IVec3, Vec3};

//= INVENTORY =====================================================================================

pub static INVENTORY: &[Voxel] = &[
    Voxel::STONE,
    Voxel::DIRT,
    Voxel::GRASS,
    Voxel::SNOW,
    Voxel::DEAD_GRASS,
    Voxel::MOIST_GRASS,
    Voxel::SAND,
    Voxel::MUD,
    Voxel::CLAY,
    Voxel::FIRE,
    Voxel::MAGMA,
    Voxel::WATER,
    Voxel::OAK_WOOD,
    Voxel::OAK_LEAVES,
    Voxel::BIRCH_WOOD,
    Voxel::BIRCH_LEAVES,
    Voxel::SPRUCE_WOOD,
    Voxel::SPRUCE_LEAVES,
    Voxel::CACTUS,
    Voxel::GOLD,
    Voxel::MIRROR,
    Voxel::BRIGHT,
];

#[derive(Default)]
pub struct UpdateResult {
    pub hit_result: Option<HitResult>,
    pub world_changed: bool,
    pub player_moved: bool,
}

pub struct GameState {
    pub player: Player,
    pub inv_sel: u8,

    pub settings: Settings,
    pub world_gen: WorldGen,
    pub voxel_materials: Vec<Material>,
}

impl GameState {
    pub fn new(world: &mut World, renderer: &Renderer) -> Self {
        let player = Player::new(
            Vec3::new(world.size as f32 * 0.5, 100.0, world.size as f32 * 0.5),
            0.3, // was 0.2
        );

        let mut settings = Settings::default();
        settings.max_ray_bounces = 4;
        settings.samples_per_pixel = 1;
        settings.sun_intensity = 4.0;
        settings.sky_color = [0.81, 0.93, 1.0];
        settings.sun_pos = Vec3::new(
            0.0f32.to_radians().sin() * 500.0,
            0.0f32.to_radians().cos() * 500.0,
            world.size as f32 * 0.5,
        )
        .to_array();

        let world_gen = WorldGen::new(fastrand::i64(..));
        world_gen.populate(IVec3::ZERO, IVec3::splat(world.size as i32), world);

        renderer.write_nodes(0, world.nodes());
        renderer.write_settings(&settings);
        renderer.write_world_data(&WorldData::new(world.min, world.size));
        let voxel_materials = VOXEL_MATERIALS.to_vec();
        renderer.write_voxel_materials(0, &voxel_materials);

        Self {
            player,
            inv_sel: 0,

            settings,
            world_gen,
            voxel_materials,
        }
    }

    pub fn update(
        &mut self,
        window: &Window,
        mut world: &mut World,
        renderer: &mut Renderer,
    ) -> UpdateResult {
        let mut output = UpdateResult::default();

        let root = world.get_node(0);
        let node = move |x: u32, y: u32, z: u32| root.get_child(x + y * 2 + z * 4);
        let edge_dist = 30;

        let mut world_moved = false;
        if (self.player.position.x as i32) < world.min().x + edge_dist {
            world_moved = true;
            world.swap_nodes(node(0, 0, 0), node(1, 0, 0));
            world.swap_nodes(node(0, 0, 1), node(1, 0, 1));
            world.swap_nodes(node(0, 1, 0), node(1, 1, 0));
            world.swap_nodes(node(0, 1, 1), node(1, 1, 1));

            world.clear_node(node(0, 0, 0));
            world.clear_node(node(0, 0, 1));
            world.clear_node(node(0, 1, 0));
            world.clear_node(node(0, 1, 1));

            world.min.x -= world.size as i32 / 2;
            self.world_gen.populate(
                world.min(),
                world.max() - IVec3::X * world.size as i32 / 2,
                &mut world,
            );
        }
        if (self.player.position.x as i32) > world.max().x - edge_dist {
            world_moved = true;
            world.swap_nodes(node(1, 0, 0), node(0, 0, 0));
            world.swap_nodes(node(1, 0, 1), node(0, 0, 1));
            world.swap_nodes(node(1, 1, 0), node(0, 1, 0));
            world.swap_nodes(node(1, 1, 1), node(0, 1, 1));

            world.clear_node(node(1, 0, 0));
            world.clear_node(node(1, 0, 1));
            world.clear_node(node(1, 1, 0));
            world.clear_node(node(1, 1, 1));

            world.min.x += world.size as i32 / 2;
            self.world_gen.populate(
                world.min() + IVec3::X * world.size as i32 / 2,
                world.max(),
                &mut world,
            );
        }
        if (self.player.position.z as i32) < world.min().z + edge_dist {
            world_moved = true;
            world.swap_nodes(node(0, 0, 0), node(0, 0, 1));
            world.swap_nodes(node(0, 1, 0), node(0, 1, 1));
            world.swap_nodes(node(1, 0, 0), node(1, 0, 1));
            world.swap_nodes(node(1, 1, 0), node(1, 1, 1));

            world.clear_node(node(0, 0, 0));
            world.clear_node(node(0, 1, 0));
            world.clear_node(node(1, 0, 0));
            world.clear_node(node(1, 1, 0));

            world.min.z -= world.size as i32 / 2;
            self.world_gen.populate(
                world.min(),
                world.max() - IVec3::Z * world.size as i32 / 2,
                &mut world,
            );
        }
        if (self.player.position.z as i32) > world.max().z - edge_dist {
            world_moved = true;
            world.swap_nodes(node(0, 0, 1), node(0, 0, 0));
            world.swap_nodes(node(0, 1, 1), node(0, 1, 0));
            world.swap_nodes(node(1, 0, 1), node(1, 0, 0));
            world.swap_nodes(node(1, 1, 1), node(1, 1, 0));

            world.clear_node(node(0, 0, 1));
            world.clear_node(node(0, 1, 1));
            world.clear_node(node(1, 0, 1));
            world.clear_node(node(1, 1, 1));

            world.min.z += world.size as i32 / 2;
            self.world_gen.populate(
                world.min() + IVec3::Z * world.size as i32 / 2,
                world.max(),
                &mut world,
            );
        }
        if world_moved {
            renderer.write_nodes(0, world.nodes());
            renderer.write_world_data(&WorldData::new(world.min, world.size));
        }

        let prev_pos = self.player.position;
        let prev_rot = self.player.rotation;
        self.player.update(window, &world);
        if window.get_input_state(InputKind::InventoryPrev).is_some() {
            self.inv_sel = (self.inv_sel as i8 - 1).clamp(0, INVENTORY.len() as i8 - 1) as u8;
        } else if window.get_input_state(InputKind::InventoryNext).is_some() {
            self.inv_sel = (self.inv_sel as i8 + 1).clamp(0, INVENTORY.len() as i8 - 1) as u8;
        }

        if prev_pos != self.player.position || prev_rot != self.player.rotation {
            output.player_moved = true;
        }

        let surface_size = renderer.surface_size();
        let camera = self.player.create_camera(surface_size);
        renderer.write_camera(&camera);

        let hit_result = self.player.cast_ray(&world);

        enum Action {
            Place,
            Break,
        }
        let action = if window.get_input_state(InputKind::GetVoxel).is_some() {
            Some(Action::Break)
        } else if window.get_input_state(InputKind::PutVoxel).is_some() {
            Some(Action::Place)
        } else {
            None
        };

        let set_vox = match action {
            Some(Action::Break) => Some(Voxel::AIR),
            Some(Action::Place) => Some(INVENTORY[self.inv_sel as usize]),
            None => None,
        };

        let set_pos = match (action, hit_result) {
            (Some(Action::Break), Some(hit)) => Some(hit.pos),
            (Some(Action::Place), Some(hit)) => Some(hit.pos + hit.face),
            _ => None,
        };

        if let (Some(pos), Some(vox)) = (set_pos, set_vox) {
            for NodeSeq { idx, count } in world.set_voxel(pos, vox).unwrap() {
                renderer.write_nodes(
                    idx as u64,
                    &world.nodes()[idx as usize..idx as usize + count as usize],
                );
            }

            renderer.reset_frame_counter();
        }

        if window.get_input_state(InputKind::InventoryNext).is_some()
            && (self.inv_sel as usize) < INVENTORY.len() - 1
        {
            self.inv_sel += 1;
        }
        if window.get_input_state(InputKind::InventoryPrev).is_some() && self.inv_sel > 0 {
            self.inv_sel -= 1;
        }

        output.hit_result = hit_result;
        output
    }
}
