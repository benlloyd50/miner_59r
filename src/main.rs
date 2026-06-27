use bracket_lib::prelude::*;
use hecs::*;

struct State {
    pub world: World,
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print(1, 1, "hello");

        handle_input(&mut self.world);

        render_state(&self.world, ctx);
    }
}

fn render_state(world: &World, ctx: &mut BTerm) {
    for pos in world.query::<&Position>().iter() {
        ctx.print_color(
            pos.x,
            pos.y,
            RGB::from_f32(1.0, 1.0, 0.0),
            RGB::from_f32(0., 0., 0.),
            "@",
        );
    }

    ctx.draw_hollow_box(
        1,
        (CONSOLE_TILES_Y / 2) - 1,
        CONSOLE_TILES_X - 3,
        (CONSOLE_TILES_Y / 2) - 3,
        SANDY_BROWN,
        BLACK,
    );
    ctx.draw_hollow_box(
        1,
        1,
        CONSOLE_TILES_X - 3,
        (CONSOLE_TILES_Y / 2) - 3,
        SANDY_BROWN,
        BLACK,
    );
}

// todo: handle input should return some sort of command
fn handle_input(world: &mut World) {
    let input = INPUT.lock();
    if input.is_key_pressed(VirtualKeyCode::Escape) {
        std::process::exit(0);
    }

    let dt = if input.is_key_pressed(VirtualKeyCode::W) {
        (0, -1)
    } else if input.is_key_pressed(VirtualKeyCode::A) {
        (-1, 0)
    } else if input.is_key_pressed(VirtualKeyCode::D) {
        (1, 0)
    } else if input.is_key_pressed(VirtualKeyCode::S) {
        (0, 1)
    } else {
        (0, 0)
    };

    if dt != (0, 0) {
        let mut pos_q = world.query::<&mut Position>();
        for pos in pos_q.iter() {
            pos.x = pos.x.saturating_add_signed(dt.0);
            pos.y = pos.y.saturating_add_signed(dt.1);
            println!("moved: {pos:?}");
        }
    }
}

// const CONSOLE_TILES_X: u32 = 640 / 8;
// const CONSOLE_TILES_Y: u32 = 200 / 8;
const CONSOLE_TILES_X: u32 = 88;
const CONSOLE_TILES_Y: u32 = 54;
const SCREEN_DIMENSIONS_X: u32 = CONSOLE_TILES_X * 8 / 4;
const SCREEN_DIMENSIONS_Y: u32 = CONSOLE_TILES_Y * 8 / 4;
const FONT_PATH: &'static str = "Anikki_Square_8x8.png";

fn main() -> BError {
    let context = BTermBuilder::new()
        .with_simple_console(CONSOLE_TILES_X, CONSOLE_TILES_Y, FONT_PATH)
        .with_fullscreen(false)
        .with_dimensions(SCREEN_DIMENSIONS_X, SCREEN_DIMENSIONS_Y)
        .with_font(FONT_PATH, 8u32, 8u32)
        .with_title("miner_59r")
        .build()?;

    let world = World::new();
    let mut gs = State { world };
    init_world(&mut gs);

    main_loop(context, gs)
}

fn init_world(state: &mut State) {
    state.world.spawn((Position { x: 1, y: 1 },));
}

#[derive(Debug)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}
