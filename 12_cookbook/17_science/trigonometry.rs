fn calc_distance() {
    let eart_r_km = 6371_f64;
    let (paris_lat_degrees, paris_long_degrees) = (48.85341_f64, -2.34880_f64);
    let (london_lat_degrees, london_long_degrees) = (51.50853_f64, -0.12574_f64);

    let paris_lat = paris_lat_degrees.to_radians();
    let london_lat = london_lat_degrees.to_radians();

    let delta_lat = (paris_lat_degrees - london_lat_degrees).to_radians();
    let delat_long = (paris_long_degrees - london_long_degrees).to_radians();

    let central_angle = (delta_lat / 2.).sin().powi(2)
        + paris_lat.cos() * london_lat.cos() * (delat_long / 2.).sin().powi(2);
    let central_angle = 2.0 * central_angle.sqrt().asin();

    let d = eart_r_km * central_angle;
    println!("Distance between Paris and London on the surface of Earth is {d:.1} kilometers");
}

fn main() {
    let x = 6_f64;
    assert_eq!(x.tan(), x.sin() / x.cos());
    calc_distance();
}
