use super::gl::{
    self,
    types::{GLenum, GLint, GLuint},
};

pub struct CharShaderProgram {
    pub id: GLuint,
    pub projection: GLint,
    pub cell_dim: GLint,
    pub background: GLint,
}

impl CharShaderProgram {
    pub fn new(shader_v: &str, shader_f: &str) -> Self {
        let shader_v = create_shader(gl::VERTEX_SHADER, shader_v);
        let shader_f = create_shader(gl::FRAGMENT_SHADER, shader_f);
        let id = create_program(shader_v, shader_f);

        unsafe {
            gl::DeleteShader(shader_v);
            gl::DeleteShader(shader_f);
            gl::UseProgram(id);
        }

        let (projection, cell_dim, background) = unsafe {
            (
                gl::GetUniformLocation(id, b"projection\0".as_ptr() as *const _),
                gl::GetUniformLocation(id, b"cellDim\0".as_ptr() as *const _),
                gl::GetUniformLocation(id, b"backgroundPass\0".as_ptr() as *const _),
            )
        };

        // TODO: assert valid uniform

        unsafe { gl::UseProgram(0) };

        CharShaderProgram { id, projection, cell_dim, background }
    }
}

impl Drop for CharShaderProgram {
    fn drop(&mut self) {
        unsafe { gl::DeleteProgram(self.id) };
    }
}

pub fn create_program(shader_v: GLuint, shader_f: GLuint) -> GLuint {
    unsafe {
        let program = gl::CreateProgram();
        gl::AttachShader(program, shader_v);
        gl::AttachShader(program, shader_f);
        gl::LinkProgram(program);

        let mut success = 0;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);

        program
    }
}

pub fn create_shader(kind: GLenum, source: &str) -> GLuint {
    let len = [source.len() as GLint];

    let shader = unsafe {
        let shader = gl::CreateShader(kind);

        gl::ShaderSource(shader, 1, &(source.as_ptr() as *const _), len.as_ptr());
        gl::CompileShader(shader);

        shader
    };

    let mut success = 0;
    unsafe { gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success) };

    shader
}
