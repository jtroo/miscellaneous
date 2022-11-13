use svg::node::element::path::{Command, Data, Position};
use svg::parser::Event;

fn main() {
    let path = "roads.svg";
    let mut content = String::new();

    // used to calculate distance of other roads
    const RED_ROAD_DISTANCE_KM: f64 = 5.0;
    let mut red_reference_road_pixel_distance: f64 = 0.0;

    let mut blue_road_distances = vec![];
    let mut green_road_distances = vec![];

    for event in svg::open(path, &mut content).unwrap() {
        match event {
            Event::Tag(_, _, attributes) => {
                let Some(data) = attributes.get("d") else { continue };
                let Some(style) = attributes.get("style") else { continue };
                let data = Data::parse(data).unwrap();
                if style.contains("stroke:#ff0000") { // red
                    if red_reference_road_pixel_distance != 0.0 {
                        panic!("multiple red lines");
                    }
                    red_reference_road_pixel_distance = data_pixel_distance(&data);
                } else if style.contains("stroke:#0000ff") { // blue
                    blue_road_distances.push(data_pixel_distance(&data));
                } else if style.contains("stroke:#008000") { // green
                    green_road_distances.push(data_pixel_distance(&data));
                } else {
                    panic!("unknown style: {style}");
                }
            }
            _ => {}
        }
    }
    if red_reference_road_pixel_distance == 0.0 {
        panic!("no reference red line found")
    }

    let pixels_per_km = red_reference_road_pixel_distance / RED_ROAD_DISTANCE_KM;
    let blue_km = blue_road_distances.iter().sum::<f64>() / pixels_per_km;
    let green_km = green_road_distances.iter().sum::<f64>() / pixels_per_km;
    println!("blue km: {blue_km}, green km: {green_km}");
}

fn data_pixel_distance(d: &Data) -> f64 {
    let cmds = d.iter().collect::<Vec<_>>();
    // expect only 1 move for every line drawn
    if cmds.len() != 1 {
        panic!("found data with cmd length more than 1: {cmds:?}");
    }
    let Command::Move(pos, params) = cmds[0] else {
        panic!("non-move command found");
    };
    // expect pairs of coordinates
    assert!(params.len() % 2 == 0);
    let points = params.chunks(2).map(|chunk| {
        Point ( f64::from(chunk[0]),  f64::from(chunk[1]))
    }).collect::<Vec<_>>();

    match *pos {
        Position::Relative => points_pixel_distance_relative(&points),
        Position::Absolute => points_pixel_distance_absolute(&points),
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct Point (f64, f64);

fn points_pixel_distance_relative(points: &[Point]) -> f64 {
    // skip the first; it's the absolute start point:
    // https://www.w3.org/TR/SVG11/paths.html#PathDataMovetoCommands
    let points = points.iter().copied().skip(1);
    let mut distance = 0.0;
    for point in points {
        let Point(x, y) = point;
        distance += f64::sqrt(x*x + y*y);
    }
    distance
}

fn points_pixel_distance_absolute(points: &[Point]) -> f64 {
    let mut points = points.iter().copied();
    let mut distance = 0.0;
    let mut prev_point = points.next().unwrap();
    for point in points {
        let Point(x, y) = point;
        let Point(x_prev, y_prev) = prev_point;
        distance += f64::sqrt((x_prev - x).powi(2) + (y_prev-y).powi(2));
        prev_point = point;
    }
    distance
}
