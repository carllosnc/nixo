use std::ffi::{CStr, c_char, c_int, c_void};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

use libloading::Library;
use nixo_core::NixoError;
use tracing::info;

#[repr(C)]
pub(super) struct MpvHandle {
    _private: [u8; 0],
}

#[repr(C)]
pub(super) struct MpvRenderContext {
    _private: [u8; 0],
}

#[repr(C)]
pub(super) struct MpvRenderParam {
    pub(super) type_: c_int,
    pub(super) data: *mut c_void,
}

pub(super) const MPV_FORMAT_FLAG: c_int = 3;
pub(super) const MPV_FORMAT_INT64: c_int = 4;
pub(super) const MPV_FORMAT_DOUBLE: c_int = 5;

pub(super) const MPV_RENDER_PARAM_INVALID: c_int = 0;
pub(super) const MPV_RENDER_PARAM_API_TYPE: c_int = 1;
pub(super) const MPV_RENDER_PARAM_BLOCK_FOR_TARGET_TIME: c_int = 12;
pub(super) const MPV_RENDER_PARAM_SW_SIZE: c_int = 17;
pub(super) const MPV_RENDER_PARAM_SW_FORMAT: c_int = 18;
pub(super) const MPV_RENDER_PARAM_SW_STRIDE: c_int = 19;
pub(super) const MPV_RENDER_PARAM_SW_POINTER: c_int = 20;

pub(super) const MPV_RENDER_UPDATE_FRAME: u64 = 1;

pub(super) type MpvRenderUpdateFn = unsafe extern "C" fn(*mut c_void);

pub(super) type FnMpvCreate = unsafe extern "C" fn() -> *mut MpvHandle;
pub(super) type FnMpvInitialize = unsafe extern "C" fn(*mut MpvHandle) -> c_int;
pub(super) type FnMpvTerminateDestroy = unsafe extern "C" fn(*mut MpvHandle);
pub(super) type FnMpvCommand = unsafe extern "C" fn(*mut MpvHandle, *const *const c_char) -> c_int;
pub(super) type FnMpvSetOptionString =
    unsafe extern "C" fn(*mut MpvHandle, *const c_char, *const c_char) -> c_int;
pub(super) type FnMpvSetProperty =
    unsafe extern "C" fn(*mut MpvHandle, *const c_char, c_int, *const c_void) -> c_int;
pub(super) type FnMpvGetProperty =
    unsafe extern "C" fn(*mut MpvHandle, *const c_char, c_int, *mut c_void) -> c_int;
pub(super) type FnMpvSetPropertyString =
    unsafe extern "C" fn(*mut MpvHandle, *const c_char, *const c_char) -> c_int;
pub(super) type FnMpvGetPropertyString =
    unsafe extern "C" fn(*mut MpvHandle, *const c_char) -> *mut c_char;
pub(super) type FnMpvFree = unsafe extern "C" fn(*mut c_void);
pub(super) type FnMpvRenderContextCreate =
    unsafe extern "C" fn(*mut *mut MpvRenderContext, *mut MpvHandle, *mut MpvRenderParam) -> c_int;
pub(super) type FnMpvRenderContextSetUpdateCallback =
    unsafe extern "C" fn(*mut MpvRenderContext, Option<MpvRenderUpdateFn>, *mut c_void);
pub(super) type FnMpvRenderContextUpdate = unsafe extern "C" fn(*mut MpvRenderContext) -> u64;
pub(super) type FnMpvRenderContextRender =
    unsafe extern "C" fn(*mut MpvRenderContext, *mut MpvRenderParam) -> c_int;
pub(super) type FnMpvRenderContextFree = unsafe extern "C" fn(*mut MpvRenderContext);

pub(super) unsafe extern "C" fn on_mpv_render_update(ctx: *mut c_void) {
    if !ctx.is_null() {
        // SAFETY: `ctx` aponta para um `AtomicBool` mantido vivo por `Arc<AtomicBool>` durante toda a vida do contexto de renderização.
        let flag = unsafe { &*(ctx as *const AtomicBool) };
        flag.store(true, Ordering::Release);
    }
}

pub(super) struct MpvApi {
    _lib: Library,
    pub(super) create: FnMpvCreate,
    pub(super) initialize: FnMpvInitialize,
    pub(super) terminate_destroy: FnMpvTerminateDestroy,
    pub(super) command: FnMpvCommand,
    pub(super) set_option_string: FnMpvSetOptionString,
    pub(super) set_property: FnMpvSetProperty,
    pub(super) get_property: FnMpvGetProperty,
    pub(super) set_property_string: FnMpvSetPropertyString,
    pub(super) get_property_string: FnMpvGetPropertyString,
    pub(super) free: FnMpvFree,
    pub(super) render_context_create: FnMpvRenderContextCreate,
    pub(super) render_context_set_update_callback: FnMpvRenderContextSetUpdateCallback,
    pub(super) render_context_update: FnMpvRenderContextUpdate,
    pub(super) render_context_render: FnMpvRenderContextRender,
    pub(super) render_context_free: FnMpvRenderContextFree,
}

impl MpvApi {
    fn candidate_paths() -> Vec<PathBuf> {
        let mut list = Vec::new();

        if cfg!(target_os = "windows") {
            if let Ok(exe) = std::env::current_exe()
                && let Some(dir) = exe.parent()
            {
                list.push(dir.join("libmpv-2.dll"));
            }
            if let Ok(cwd) = std::env::current_dir() {
                list.push(cwd.join("libmpv-2.dll"));
                list.push(cwd.join("vendor").join("mpv").join("libmpv-2.dll"));
            }
            let workspace_vendor = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join("..")
                .join("vendor")
                .join("mpv")
                .join("libmpv-2.dll");
            list.push(workspace_vendor);
            list.push(PathBuf::from("libmpv-2.dll"));
            list.push(PathBuf::from("mpv-2.dll"));
            list.push(PathBuf::from("mpv-1.dll"));
        } else if cfg!(target_os = "macos") {
            list.push(PathBuf::from("libmpv.2.dylib"));
            list.push(PathBuf::from("libmpv.dylib"));
            list.push(PathBuf::from("/opt/homebrew/lib/libmpv.dylib"));
            list.push(PathBuf::from("/usr/local/lib/libmpv.dylib"));
        } else {
            list.push(PathBuf::from("libmpv.so.2"));
            list.push(PathBuf::from("libmpv.so"));
            list.push(PathBuf::from("libmpv.so.1"));
        }

        list
    }

    pub(super) fn load() -> Result<Self, NixoError> {
        let candidates = Self::candidate_paths();

        let mut last_err = String::new();
        for candidate in candidates {
            // SAFETY: Carregamento dinâmico da biblioteca compartilhada oficial do libmpv.
            match unsafe { Library::new(&candidate) } {
                Ok(lib) => {
                    // SAFETY: Resolução dos símbolos C exportados pela ABI estável do libmpv.
                    unsafe {
                        let create = *lib
                            .get::<FnMpvCreate>(b"mpv_create\0")
                            .map_err(|e| NixoError::InitError(e.to_string()))?;
                        let initialize = *lib
                            .get::<FnMpvInitialize>(b"mpv_initialize\0")
                            .map_err(|e| NixoError::InitError(e.to_string()))?;
                        let terminate_destroy = *lib
                            .get::<FnMpvTerminateDestroy>(b"mpv_terminate_destroy\0")
                            .map_err(|e| NixoError::InitError(e.to_string()))?;
                        let command = *lib
                            .get::<FnMpvCommand>(b"mpv_command\0")
                            .map_err(|e| NixoError::InitError(e.to_string()))?;
                        let set_option_string = *lib
                            .get::<FnMpvSetOptionString>(b"mpv_set_option_string\0")
                            .map_err(|e| NixoError::InitError(e.to_string()))?;
                        let set_property = *lib
                            .get::<FnMpvSetProperty>(b"mpv_set_property\0")
                            .map_err(|e| NixoError::InitError(e.to_string()))?;
                        let get_property = *lib
                            .get::<FnMpvGetProperty>(b"mpv_get_property\0")
                            .map_err(|e| NixoError::InitError(e.to_string()))?;
                        let set_property_string = *lib
                            .get::<FnMpvSetPropertyString>(b"mpv_set_property_string\0")
                            .map_err(|e| NixoError::InitError(e.to_string()))?;
                        let get_property_string = *lib
                            .get::<FnMpvGetPropertyString>(b"mpv_get_property_string\0")
                            .map_err(|e| NixoError::InitError(e.to_string()))?;
                        let free = *lib
                            .get::<FnMpvFree>(b"mpv_free\0")
                            .map_err(|e| NixoError::InitError(e.to_string()))?;
                        let render_context_create = *lib
                            .get::<FnMpvRenderContextCreate>(b"mpv_render_context_create\0")
                            .map_err(|e| NixoError::InitError(e.to_string()))?;
                        let render_context_set_update_callback = *lib
                            .get::<FnMpvRenderContextSetUpdateCallback>(
                                b"mpv_render_context_set_update_callback\0",
                            )
                            .map_err(|e| NixoError::InitError(e.to_string()))?;
                        let render_context_update = *lib
                            .get::<FnMpvRenderContextUpdate>(b"mpv_render_context_update\0")
                            .map_err(|e| NixoError::InitError(e.to_string()))?;
                        let render_context_render = *lib
                            .get::<FnMpvRenderContextRender>(b"mpv_render_context_render\0")
                            .map_err(|e| NixoError::InitError(e.to_string()))?;
                        let render_context_free = *lib
                            .get::<FnMpvRenderContextFree>(b"mpv_render_context_free\0")
                            .map_err(|e| NixoError::InitError(e.to_string()))?;

                        info!(
                            "Biblioteca dinâmica libmpv carregada com sucesso: {}",
                            candidate.display()
                        );
                        return Ok(Self {
                            _lib: lib,
                            create,
                            initialize,
                            terminate_destroy,
                            command,
                            set_option_string,
                            set_property,
                            get_property,
                            set_property_string,
                            get_property_string,
                            free,
                            render_context_create,
                            render_context_set_update_callback,
                            render_context_update,
                            render_context_render,
                            render_context_free,
                        });
                    }
                }
                Err(e) => {
                    last_err = e.to_string();
                }
            }
        }

        Err(NixoError::InitError(format!(
            "Não foi possível encontrar libmpv no sistema ({last_err})"
        )))
    }

    pub(super) unsafe fn set_opt(&self, handle: *mut MpvHandle, key: &CStr, val: &CStr) {
        // SAFETY: `handle` é válido e `key`/`val` são strings C válidas.
        unsafe {
            let _ = (self.set_option_string)(handle, key.as_ptr(), val.as_ptr());
        }
    }
}

// SAFETY: MpvApi armazena apenas o handle da DLL e ponteiros de funções C thread-safe do libmpv.
unsafe impl Send for MpvApi {}
// SAFETY: Ponteiros de funções C do libmpv podem ser chamados concorrentemente em handles distintos.
unsafe impl Sync for MpvApi {}
