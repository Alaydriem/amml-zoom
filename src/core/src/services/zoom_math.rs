pub struct ZoomMath;

impl ZoomMath {
    // log-space interpolation: each equal step in level multiplies the magnification by the same amount
    pub fn zoomed_fov(base_fov: f32, zoom_factor: f32, level: f32) -> f32 {
        base_fov / zoom_factor.powf(level)
    }

    pub fn blended_base(normal_fov: f32, steady_fov: f32, level: f32, fade: bool) -> f32 {
        if fade { normal_fov + (steady_fov - normal_fov) * level } else { normal_fov }
    }

    // the ratio that keeps the world under the crosshair moving at the same on-screen speed
    pub fn turn_scale(base_fov: f32, zoomed_fov: f32, strength: f32) -> f32 {
        let ratio = (zoomed_fov.to_radians() / 2.0).tan() / (base_fov.to_radians() / 2.0).tan();
        1.0 + strength * (ratio - 1.0)
    }
}
