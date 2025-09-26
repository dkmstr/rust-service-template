use windows::{
    Win32::Foundation::*, Win32::System::Services::*, Win32::System::Threading::*, core::*,
};

use crate::launcher;

const SERVICE_NAME: PCWSTR = w!("RustExampleService");

#[derive(Clone, Debug)]
struct ServiceContext {
    status_handle: SERVICE_STATUS_HANDLE,
    stop_event: HANDLE,
    async_stop: std::sync::Arc<tokio::sync::Notify>,
}

unsafe impl Send for ServiceContext {}
unsafe impl Sync for ServiceContext {}

impl ServiceContext {
    fn new() -> Self {
        Self {
            status_handle: SERVICE_STATUS_HANDLE(std::ptr::null_mut()),
            stop_event: unsafe { CreateEventW(None, true, false, None).unwrap() },
            async_stop: std::sync::Arc::new(tokio::sync::Notify::new()),
        }
    }

    pub fn report_status(
        &self,
        current_state: SERVICE_STATUS_CURRENT_STATE,
        checkpoint: u32,
        wait_hint_ms: u32,
    ) -> Result<()> {
        unsafe {
            let status = SERVICE_STATUS {
                dwServiceType: SERVICE_USER_OWN_PROCESS,
                dwCurrentState: current_state,
                dwControlsAccepted: match current_state {
                    SERVICE_RUNNING => SERVICE_ACCEPT_STOP,
                    SERVICE_STOP_PENDING => SERVICE_ACCEPT_STOP,
                    _ => 0,
                },
                dwWin32ExitCode: NO_ERROR.0,
                dwServiceSpecificExitCode: 0,
                dwCheckPoint: checkpoint,
                dwWaitHint: wait_hint_ms,
            };

            SetServiceStatus(self.status_handle, &status)?;
        }
        Ok(())
    }

    pub fn stop(&self) -> Result<()> {
        unsafe {
            SetEvent(self.stop_event)?;
        }
        Ok(())
    }

    pub fn wait_for_stop(&self, timeout_ms: u32) -> Result<WAIT_EVENT> {
        unsafe {
            let res = WaitForSingleObject(self.stop_event, timeout_ms);
            Ok(res)
        }
    }

    pub fn close(&self) -> Result<()> {
        if !self.stop_event.is_invalid() {
            unsafe {
                CloseHandle(self.stop_event)?;
            }
        }
        Ok(())
    }
}

impl Default for ServiceContext {
    fn default() -> Self {
        Self::new()
    }
}

// Firma del handler OK; devuelve DWORD
extern "system" fn service_handler(
    ctrl: u32,
    _event_type: u32,
    _event_data: *mut std::ffi::c_void,
    _context: *mut std::ffi::c_void,
) -> u32 {
    let ctx = unsafe { &mut *(_context as *mut ServiceContext) };

    match ctrl {
        SERVICE_CONTROL_STOP => {
            // Lanzamos un thread que hace el trabajo y va notificando progreso
            let mut checkpoint = 1;
            ctx.async_stop.notify_waiters();
            while ctx.wait_for_stop(100).unwrap() == WAIT_TIMEOUT {
                std::thread::sleep(std::time::Duration::from_millis(100));
                // Avisar al SCM que seguimos en STOP_PENDING
                let _ = ctx.report_status(SERVICE_STOP_PENDING, checkpoint, 10000);
                checkpoint += 1;
            }
        }
        SERVICE_CONTROL_INTERROGATE => {
            let _ = ctx.report_status(SERVICE_RUNNING, 0, 0);
        }
        _ => {}
    }
    NO_ERROR.0
}

extern "system" fn service_main(_argc: u32, _argv: *mut PWSTR) {
    unsafe {
        // Register the service control handler, with our context
        let mut ctx = ServiceContext::new();

        let ctx_ptr: *mut ServiceContext = &mut ctx;
        ctx.status_handle = match RegisterServiceCtrlHandlerExW(
            SERVICE_NAME,
            Some(service_handler),
            Some(ctx_ptr as *mut _),
        ) {
            Ok(h) => h,
            Err(_) => return,
        };

        let _ = ctx.report_status(SERVICE_START_PENDING, 0, 3000);

        // Something here to initialize the service...

        let _ = ctx.report_status(SERVICE_RUNNING, 0, 0);

        // Launch a thread that does some work and then signals the stop event
        let ctx_thread = ctx.clone();
        std::thread::spawn(move || {
            // Execute async work
            launcher::run(ctx_thread.async_stop.clone());
            // When done, signal the service to stop
            ctx_thread.stop().unwrap();
        });

        // Wait until the stop event is signaled
        let _ = ctx.wait_for_stop(INFINITE);

        let _ = ctx.report_status(SERVICE_STOPPED, 0, 0);
        let _ = ctx.close();
    }
}

pub fn run_service() -> Result<()> {
    // Service Table: StartServiceCtrlDispatcherW(*const SERVICE_TABLE_ENTRYW) -> Result<()>
    let table = [
        SERVICE_TABLE_ENTRYW {
            lpServiceName: PWSTR(SERVICE_NAME.0 as *mut _),
            lpServiceProc: Some(service_main),
        },
        SERVICE_TABLE_ENTRYW {
            lpServiceName: PWSTR::null(),
            lpServiceProc: None,
        },
    ];

    unsafe {
        StartServiceCtrlDispatcherW(table.as_ptr())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stop_signals_notify_and_event() {
        let mut ctx = ServiceContext::new();
        ctx.status_handle = SERVICE_STATUS_HANDLE::default(); // dummy

        let ctx_ptr: *mut ServiceContext = &mut ctx;

        // Set up a thread to wait for the notify
        let notify = ctx.async_stop.clone();
        
        let ctx_clone = ctx.clone();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                notify.notified().await;
                ctx_clone.stop().unwrap();
            });
        });

        // Wait a bit to ensure the thread is waiting
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Simulate the service control handler receiving a STOP command
        service_handler(
            SERVICE_CONTROL_STOP,
            0,
            std::ptr::null_mut(),
            ctx_ptr as *mut _,
        );

        // We only reach here if stop is called
        if ctx.wait_for_stop(1000).unwrap() != WAIT_OBJECT_0 {
            panic!("Stop event was not signaled");
        }

        let _ = ctx.close();
    }
}
