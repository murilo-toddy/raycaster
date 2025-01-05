use raylib::prelude::*;

fn main() {
    let width = 400;
    let height = 400;

    let (mut rl,  thread) = raylib::init()
        .size(width, height)
        .title("raycast")
        .build();

    rl.set_target_fps(60);

    let grid = [
        [0, 1, 0, 0, 0, 0],
        [0, 0, 1, 0, 0, 0],
        [0, 0, 0, 1, 0, 0],
        [0, 0, 0, 0, 1, 0],
        [0, 0, 0, 0, 0, 1],
        [0, 0, 0, 0, 0, 0],
    ];

    let rows = grid.len();
    let cols = grid[0].len();

    let cell_width = width / cols as i32;
    let cell_height = height / rows as i32;

    while !rl.window_should_close() {
        let mut drawing = rl.begin_drawing(&thread);
        drawing.clear_background(Color::WHITE);

        // draw grid
        for row in 1..rows as i32 {
            drawing.draw_line(0, cell_height * row, width, cell_height * row, Color::BLACK);
        }
        for col in 1..cols as i32 {
            drawing.draw_line(cell_width * col, 0, cell_width * col, height, Color::BLACK);
        }

        for (row, cells) in grid.iter().enumerate() {
            for (col, cell) in cells.iter().enumerate() {
                if *cell > 0 {
                    drawing.draw_rectangle(
                        col as i32 * cell_width, row as i32 * cell_height, cell_width, cell_height, Color::BLUE);
                }
            }
        }

        let player = Vector2::new(1.3 * cell_width as f32, 4.6 * cell_height as f32);
        drawing.draw_circle(player.x as i32, player.y as i32, 10.0, Color::YELLOW);

        let dir = Vector2::new(drawing.get_mouse_x() as f32 - player.x, drawing.get_mouse_y() as f32 - player.y).normalized();

        // TODO: cleanup
        let generate_next_point = |current: Vector2| -> Option<Vector2> {
            let next_x = if dir.x > 0.0 {
               (current.x as i32 / cell_width) * cell_width + cell_width
            } else {
                if current.x as i32 % cell_width == 0 {
                    (current.x as i32 / cell_width) * cell_width - cell_width
                } else {
                    (current.x as i32 / cell_width) * cell_width
                }
            };
            let corresponding_y = current.y + ((dir.y / dir.x) * (next_x as f32 - current.x));

            let next_y = if dir.y > 0.0 {
               (current.y as i32 / cell_height) * cell_height + cell_height
            } else {
                if current.y as i32 % cell_height == 0 {
                    (current.y as i32 / cell_height) * cell_height - cell_height
                } else {
                    (current.y as i32 / cell_height) * cell_height
                }
            };
            let corresponding_x = current.x + ((dir.x / dir.y) * (next_y as f32 - current.y));

            if f32::abs(current.y - corresponding_y) < f32::abs(current.y - next_y as f32) {
                if next_x > 0 && next_x < width {
                    return Some(Vector2::new(next_x as f32, corresponding_y as f32));
                } else {
                    return None;
                }
            }

            if next_y > 0 && next_y < height {
                return Some(Vector2::new(corresponding_x as f32, next_y as f32));
            } else {
                return None;
            }
        };

        let get_cell_index = |point: Vector2| -> (usize, usize) {
            if point.y as i32 % cell_height == 0 {
                let i = point.x as i32 / cell_width;
                let j = if dir.y > 0.0 { point.y as i32 / cell_height } else { point.y as i32 / cell_height - 1 };
                return (i as usize, j as usize);
            }
            if point.x as i32 % cell_width == 0 {
                let i = if dir.x > 0.0 { point.x as i32 / cell_width } else { point.x as i32 / cell_width - 1 };
                let j = point.y as i32 / cell_height;
                return (i as usize, j as usize);
            }
            return (0, 0);
        };

        let check_collision = |i: usize, j: usize| -> bool {
            return *grid.get(j).map(|maybe_entry| maybe_entry.get(i)).flatten().unwrap_or(&0) > 0
        };

        let mut current = player;
        while let Some(point) = generate_next_point(current) {
            drawing.draw_circle(point.x as i32, point.y as i32, 3.0, Color::RED);
            drawing.draw_line_v(current, point, Color::RED);

            let (i, j) = get_cell_index(point);
            if check_collision(i, j) {
                // got collision
                drawing.draw_rectangle(cell_width * i as i32, cell_height * j as i32, cell_width, cell_height, Color::BLACK);
                break;
            }
            current = point;
        }
    }
}
