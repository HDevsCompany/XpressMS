// roles.rs: los códigos de rol y la comprobación de permisos.
// Los permisos viven aquí, en el backend: la base solo guarda qué roles tiene cada usuario.
// Cada ruta del panel dirá qué roles admite, por ejemplo: exigir_rol(&usuario, &[ROL_ADMIN]).

// Códigos del catálogo Acceso.Roles (deben coincidir con los de 04_acceso_personal.sql).
pub const ROL_ADMIN: &str = "ADM";
pub const ROL_TECNICO: &str = "TEC";
pub const ROL_VENDEDOR: &str = "VEN";

// ¿Alguno de los roles que tiene la persona está entre los permitidos?
pub fn tiene_algun_rol(roles_del_usuario: &[String], permitidos: &[&str]) -> bool {
    roles_del_usuario
        .iter()
        .any(|rol| permitidos.contains(&rol.as_str()))
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn roles(lista: &[&str]) -> Vec<String> {
        lista.iter().map(|r| r.to_string()).collect()
    }

    #[test]
    fn un_rol_permitido_alcanza() {
        assert!(tiene_algun_rol(&roles(&["TEC", "VEN"]), &[ROL_VENDEDOR]));
        assert!(tiene_algun_rol(&roles(&["ADM"]), &[ROL_ADMIN, ROL_TECNICO]));
    }

    #[test]
    fn sin_rol_permitido_se_niega() {
        assert!(!tiene_algun_rol(&roles(&["TEC"]), &[ROL_ADMIN]));
        assert!(!tiene_algun_rol(&roles(&[]), &[ROL_ADMIN]));
        assert!(!tiene_algun_rol(&roles(&["ADM"]), &[]));
    }
}
