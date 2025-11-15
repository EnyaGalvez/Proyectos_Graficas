// src/obj_loader.rs
use nalgebra_glm::Vec3;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
pub struct Face {
    pub vertex_indices: Vec<[usize; 3]>,  // Triángulos después de triangular
}

/// Carga un archivo OBJ y retorna vértices y caras
pub fn load_obj(path: &str) -> Result<(Vec<Vec3>, Vec<Face>), String> {
    let file = File::open(path)
        .map_err(|e| format!("No se pudo abrir {}: {}", path, e))?;
    
    let reader = BufReader::new(file);
    let mut vertices = Vec::new();
    let mut raw_faces: Vec<Vec<usize>> = Vec::new();
    
    for line in reader.lines() {
        let l = line.map_err(|e| format!("Error leyendo línea: {}", e))?;
        let l = l.trim();
        
        if l.is_empty() || l.starts_with('#') {
            continue;
        }
        
        if l.starts_with("v ") {
            // Vértice: v x y z
            let parts: Vec<&str> = l.split_whitespace().collect();
            if parts.len() < 4 {
                continue;
            }
            
            let x: f32 = parts[1].parse().unwrap_or(0.0);
            let y: f32 = parts[2].parse().unwrap_or(0.0);
            let z: f32 = parts[3].parse().unwrap_or(0.0);
            
            vertices.push(Vec3::new(x, y, z));
        } else if l.starts_with("f ") {
            // Cara: f v1/vt1/vn1 v2/vt2/vn2 v3/vt3/vn3
            let parts: Vec<&str> = l.split_whitespace().skip(1).collect();
            let mut indices: Vec<usize> = Vec::new();
            
            for p in parts {
                // Tomar solo el índice de vértice (antes del primer /)
                let first = p.split('/').next().unwrap_or("");
                if first.is_empty() {
                    continue;
                }
                
                if let Ok(v) = first.parse::<isize>() {
                    // OBJ usa índices 1-based, convertir a 0-based
                    let idx = if v > 0 {
                        (v - 1) as usize
                    } else {
                        // Índices negativos cuentan desde el final
                        (vertices.len() as isize + v) as usize
                    };
                    indices.push(idx);
                }
            }
            
            if indices.len() >= 3 {
                raw_faces.push(indices);
            }
        }
    }
    
    // Triangular caras (convertir polígonos a triángulos)
    let mut faces = Vec::new();
    for poly in raw_faces {
        let mut triangles = Vec::new();
        
        // Fan triangulation desde el primer vértice
        for i in 1..(poly.len() - 1) {
            triangles.push([poly[0], poly[i], poly[i + 1]]);
        }
        
        faces.push(Face {
            vertex_indices: triangles,
        });
    }
    
    Ok((vertices, faces))
}

/// Convierte vértices y caras a un array plano de triángulos
pub fn setup_vertex_array(vertices: &[Vec3], faces: &[Face]) -> Vec<Vec3> {
    let mut vertex_array = Vec::new();
    
    for face in faces {
        for tri in &face.vertex_indices {
            let v0 = vertices[tri[0]];
            let v1 = vertices[tri[1]];
            let v2 = vertices[tri[2]];
            
            vertex_array.push(v0);
            vertex_array.push(v1);
            vertex_array.push(v2);
        }
    }
    
    vertex_array
}