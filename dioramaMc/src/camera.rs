// src/camera.rs

use nalgebra_glm::{Mat4, Vec3};
use std::f32::consts::PI;

fn sf_normalize(v: Vec3) -> Vec3 {
    let n = v.magnitude();
    if n > 1e-8 { v / n } else { v }
}

// conversiones: de rad a grados y de grados a rad
fn rad_to_deg(r: f32) -> f32 { r * 180.0 / PI }
fn deg_to_rad(d: f32) -> f32 { d * PI / 180.0 }

// wrap para proteger rango de movimiento: en grados [0, 360) y rad [0, 2pi)
fn wrap_deg_360(mut d: f32) -> f32 {
    d %= 360.0;
    if d < 0.0 { d += 360.0; }
    d
}

fn wrap_rad_tau(d: f32) -> f32 {
    let tau = 2.0 * PI;
    let mut x = d % tau;
    if x < 0.0 { x += tau; }
    x
}

/* Se definen vectores y angulos de movimiento:
 * Vectores: eye (Posicion de la camara), center (Punto al que mira la camara), up
 * Angulos: YAWN (y), PITCH (z), ROLL (x)
*/
pub struct Camera {
    pub eye: Vec3,
    pub center: Vec3,
    pub up: Vec3,

    pub yawn: f32,
    pub pitch: f32,
    pub roll: f32
}

impl Camera { // Inicializacion de angulos
    pub fn new(eye: Vec3, center: Vec3, up: Vec3) -> Self {
        let r = eye - center;
        let radius = r.magnitude().max(1e-6);
        let yawn = r.z.atan2(r.x);
        let pitch = (r.y / radius).asin();
        let roll = 0.0;

        Camera {
            eye,
            center,
            up,
            yawn: wrap_rad_tau(yawn),
            pitch,
            roll,
        }
    }

    // Ejes de camara: z = forward, x = right, y = up
    pub fn axes(&self) -> (Vec3, Vec3, Vec3) {
        let z = sf_normalize(self.center - self.eye);
        let x = sf_normalize(z.cross(&self.up));
        let y = sf_normalize(x.cross(&z));
        (x, y, z)
    }

    pub fn view_matrix(&self) -> Mat4 {
        let (x, y, z) = self.axes();
        let ex = -x.dot(&self.eye);
        let ey = -y.dot(&self.eye);
        let ez = z.dot(&self.eye) * -1.0;

        Mat4::new(
            x.x, y.x, -z.x, 0.0,
            x.y, y.y, -z.y, 0.0,
            x.z, y.z, -z.z, 0.0,
            ex,  ey,   ez,  1.0,
        )
    }

    pub fn basis_change(&self, v: &Vec3) -> Vec3 {
        let (x, y, z) = self.axes();
        let rotated = v.x * x + v.y * y - v.z * z;
        sf_normalize(rotated)
    }

    // Orbita al rededor del centro por YAWN, PITCH y ROLL
    pub fn orbit(&mut self, delta_yaw: f32, delta_pitch: f32) {
        let r_vec = self.eye - self.center;
        let radius = r_vec.magnitude().max(1e-6);

        let mut yawn  = self.yawn  + delta_yaw;
        let mut pitch = self.pitch + delta_pitch;

        // envolver yawn a 360° exactos (en grados)
        let yaw_deg = wrap_deg_360(rad_to_deg(yawn));
        yawn = wrap_rad_tau(deg_to_rad(yaw_deg));

        // limitar pitch para evitar singularidad
        let pitch_limit = (PI / 2.0) - 0.001;
        if pitch >  pitch_limit { pitch =  pitch_limit; }
        if pitch < -pitch_limit { pitch = -pitch_limit; }

        let cos_p = pitch.cos();
        let new_eye_offset = Vec3::new(
            radius * yawn.cos() * cos_p,
            radius * pitch.sin(),
            radius * yawn.sin() * cos_p,
        );

        self.eye = self.center + new_eye_offset;
        self.yawn = yawn;
        self.pitch = pitch;
    }

    // Desplazamiento objetivo en el plano
    pub fn pan(&mut self, dx: f32, dy: f32) {
        let (x, y, _z) = self.axes();
        let delta = dx * x + dy * y;
        self.eye += delta;
        self.center += delta;
    }

    // Acercamiento y alejamiento de la camara
    pub fn dolly(&mut self, dz: f32) {
        let (_x, _y, z) = self.axes();
        let delta = dz * z;
        self.eye += delta;
    }
}