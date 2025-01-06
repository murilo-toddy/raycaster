use raylib::prelude::*;

#[derive(PartialEq, Eq)]
enum Side {
    X,
    Y,
}

fn get_color(cell_value: i32) -> Color {
    return match cell_value {
        1 => Color::BLUE,
        2 => Color::RED,
        3 => Color::YELLOW,
        _ => Color::BLUE,
    };
}

fn main() {
    let width = 800;
    let height = 800;

    let minimap_width = 200;
    let minimap_height = 200;
    let minimap_width_scale = minimap_width as f32 / width as f32;
    let minimap_height_scale = minimap_height as f32 / height as f32;
    let minimap_offset = Vector2::new(0.0, 0.0);

    let to_minimap_coordinates = |x: i32, y: i32| -> (i32, i32) {
        return ((x as f32 * minimap_width_scale + minimap_offset.x) as i32, (y as f32 * minimap_height_scale + minimap_offset.y) as i32);
    };

    let to_minimap_coordinates_v = |v: Vector2| -> Vector2 {
        return Vector2::new(v.x * minimap_width_scale as f32, v.y * minimap_height_scale as f32) + minimap_offset;
    };

    let (mut rl,  thread) = raylib::init()
        .size(width, height)
        .title("raycast")
        .build();

    rl.set_target_fps(60);

    let grid = [
        [0, 1, 0, 0, 0, 0],
        [0, 0, 2, 0, 0, 0],
        [3, 0, 0, 1, 0, 0],
        [1, 0, 0, 0, 3, 0],
        [2, 0, 0, 0, 0, 2],
        [1, 2, 1, 1, 1, 0],
    ];

    let rows = grid.len();
    let cols = grid[0].len();

    let cell_width = width / cols as i32;
    let cell_height = height / rows as i32;
    let mut player = Vector2::new(1.3 * cell_width as f32, 4.6 * cell_height as f32);
    let mut looking_dir = Vector2::new(width as f32 / 2.0 - player.x, height as f32 / 2.0 - player.y).normalized();

    while !rl.window_should_close() {
        let mut drawing = rl.begin_drawing(&thread);
        drawing.clear_background(Color::WHITE);

        drawing.draw_circle_v(to_minimap_coordinates_v(player), 5.0, Color::YELLOW);

        if drawing.is_key_down(KeyboardKey::KEY_W) {
            player = player + looking_dir.scale_by(1.0);
        }
        if drawing.is_key_down(KeyboardKey::KEY_S) {
            player = player - looking_dir.scale_by(1.0);
        }
        if drawing.is_key_down(KeyboardKey::KEY_A) {
            looking_dir.rotate(-0.05);
        }
        if drawing.is_key_down(KeyboardKey::KEY_D) {
            looking_dir.rotate(0.05);
        }
        
        // draw grid
        for row in 0..=rows as i32 {
            let (x1, y1) = to_minimap_coordinates(0, cell_height * row);
            let (x2, y2) = to_minimap_coordinates(width, cell_height * row);
            drawing.draw_line(x1, y1, x2, y2, Color::BLACK);
        }
        for col in 0..=cols as i32 {
            let (x1, y1) = to_minimap_coordinates(cell_width * col, 0);
            let (x2, y2) = to_minimap_coordinates(cell_width * col, height);
            drawing.draw_line(x1, y1, x2, y2, Color::BLACK);
        }

        for (row, cells) in grid.iter().enumerate() {
            for (col, cell) in cells.iter().enumerate() {
                if *cell > 0 {
                    let (x, y) = to_minimap_coordinates(col as i32 * cell_width, row as i32 * cell_height);
                    let (minimap_cell_width, minimap_cell_height) = to_minimap_coordinates(cell_width, cell_height);
                    drawing.draw_rectangle(x, y, minimap_cell_width, minimap_cell_height, get_color(*cell));
                }
            }
        }

        let generate_next_point = |current: Vector2, dir: Vector2| -> Option<Vector2> {
            fn generate_next(c: f32, dir: f32, w: i32) -> i32 {
                let next = c as i32 / w * w;
                if dir > 0.0 {
                    return next + w;
                }
                if c as i32 % w == 0 {
                    return next - w;
                }
                return next;
            }

            let check_bounds_and_return = |point: Vector2| -> Option<Vector2> {
                if point.x < 0.0 || point.x > width as f32 || point.y < 0.0 || point.y > height as f32 {
                    return None;
                }
                return Some(point);
            };

            // TODO: when hitting an edge the cell selected is wrong for dir.y < 0.0
            if current.x as i32 % cell_width == 0  && current.y as i32 % cell_height == 0 {
                return Some(Vector2::new(
                        (current.x as i32 / cell_width * cell_width) as f32 + 1.0,
                        (current.y as i32 / cell_height * cell_height) as f32,
                ));
            }

            let next_x = generate_next(current.x, dir.x, cell_width);
            let corresponding_y = current.y + ((dir.y / dir.x) * (next_x as f32 - current.x));

            let next_y = generate_next(current.y, dir.y, cell_height);
            let corresponding_x = current.x + ((dir.x / dir.y) * (next_y as f32 - current.y));

            if f32::abs(current.y - corresponding_y) < f32::abs(current.y - next_y as f32) {
                return check_bounds_and_return(Vector2::new(next_x as f32, corresponding_y as f32));
            }
            return check_bounds_and_return(Vector2::new(corresponding_x as f32, next_y as f32));
        };

        let get_cell_index_and_side = |point: Vector2, dir: Vector2| -> (usize, usize, Side) {
            fn get_corresponding_index(dir: f32, c: f32, w: i32) -> usize {
                let i = (c as i32 / w) as usize;
                return if dir > 0.0 || i == 0 { i } else { i - 1 };
            }
            if point.x as i32 % cell_width == 0 {
                let i = get_corresponding_index(dir.x, point.x, cell_width) as i32;
                let j = point.y as i32 / cell_height;
                return (i as usize, j as usize, Side::X);
            }
            if point.y as i32 % cell_height == 0 {
                let i = point.x as i32 / cell_width;
                let j = get_corresponding_index(dir.y, point.y, cell_height) as i32;
                return (i as usize, j as usize, Side::Y);
            }
            return (0, 0, Side::X);
        };

        let get_cell_value = |i: usize, j: usize| {
            return *grid.get(j).map(|maybe_entry| maybe_entry.get(i)).flatten().unwrap_or(&0);
        };

        let plane_distance = width as f32 / 800.0;
        let plane_size = width as f32 / 400.0;
        let plane_y = 1.0;
        let plane_x = - plane_y * looking_dir.y / looking_dir.x;
        let pos_plus_dir = player + looking_dir.scale_by(plane_distance);
        let plane = Vector2::new(plane_x, plane_y).normalized().scale_by(plane_size / 2.0);

        let scale = 100;
        let step = 1;
        let rows_count = 2.0 * scale as f32;
        let line_thickness = width as f32 * step as f32 / rows_count;
        for delta_plane in (-scale..=scale).step_by(step as usize) { 
            let plane_offset = delta_plane as f32 / scale as f32;  // -1..1
            let plane_point = pos_plus_dir + plane.scale_by(plane_offset);

            let direction = plane_point - player;
            let mut current = plane_point;
            while let Some(point) = generate_next_point(current, direction) {
                let (i, j, side) = get_cell_index_and_side(point, direction);
                let cell_value = get_cell_value(i as usize, j as usize);
                if cell_value > 0 {
                    // collision
                    drawing.draw_circle_v(to_minimap_coordinates_v(point), 3.0, Color::RED);
                    
                    // calculate distance between point and plane line
                    // two points on the line
                    let p1 = pos_plus_dir;
                    let p2 = pos_plus_dir + plane;
                    
                    // TODO: this expression can be simplified
                    // distance between line formed by two points (x1, y1), (x2, y2) and a point (x0, y0)
                    // d = |(y2 - y1)*x0 - (x2 - x1)*y0  + x2y1 - y2x1| / sqrt((y2 - y1) ** 2 + (x2 - x1) ** 2)
                    let numerator = f32::abs((p2.y - p1.y) * point.x - (p2.x - p1.x) * point.y + p2.x*p1.y - p2.y*p1.x);
                    let denominator = f32::sqrt((p2.y - p1.y).powi(2) + (p2.x - p1.x).powi(2));
                    let point_line_distance = numerator / denominator;

                    let line_height = height as f32 / point_line_distance * 10.0;
                    let draw_start = f32::max(-line_height / 2.0 + height as f32 / 2.0, 0.0);
                    let draw_end = f32::min(line_height / 2.0 + height as f32 / 2.0, height as f32 - 1.0);

                    // plane_offset = -1 => x = 0
                    // plane_offset =  1 => x = width
                    let mut row = width as f32 * ((plane_offset + 1.0) / 2.0);
                    if looking_dir.x < 0.0 { row = width as f32 - row; }

                    let mut color = get_color(cell_value);
                    if side == Side::Y {
                        color = color.brightness(-0.5);
                    }
                    drawing.draw_line_ex(Vector2::new(row, draw_start as f32), Vector2::new(row, draw_end as f32), line_thickness, color);

                    break;
                }
                current = point;
            }
        }
    }
}
