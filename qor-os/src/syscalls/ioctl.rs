use crate::*;

use fs::ioctl::IOControlCommand;

/// This is beyond unsafe, but this is what happens when we interact with C like
/// this
fn map_ptr<T>(proc: &mut super::Process, ptr: usize) -> &'static mut T
{
    unsafe { (proc.map_mem(ptr).unwrap() as *mut T).as_mut() }.unwrap()
}
 
/// Ioctl Syscall
pub fn syscall_ioctl(proc: &mut super::Process, fd: usize, cmd: usize, args: usize) -> usize
{
    let structured_command = 
        match cmd
        {
            /* /include/uapi/linux/fb.h - Line 14
                #define FBIOGET_VSCREENINFO	0x4600
                #define FBIOPUT_VSCREENINFO	0x4601
                #define FBIOGET_FSCREENINFO	0x4602
                #define FBIOGETCMAP	        0x4604
                #define FBIOPUTCMAP         0x4605
                #define FBIOPAN_DISPLAY		0x4606
            */
            // Framebuffer
            0x4600 =>
            {
                IOControlCommand::FrameBufferGetVariableInfo{ response: map_ptr(proc, args) }
            },
            0x4601 =>
            {
                IOControlCommand::FrameBufferPutVariableInfo{ response: map_ptr(proc, args) }
            },
            0x4602 =>
            {
                IOControlCommand::FrameBufferGetFixedInfo{ response: map_ptr(proc, args) }
            },
            0x46FF =>
            {
                IOControlCommand::FrameBufferFlush
            },

            // Real Time Clock
            0x7009 =>
            {
                IOControlCommand::RealTimeClockGetTime{ response: map_ptr(proc, args) }
            },
            0x70FF =>
            {
                IOControlCommand::RealTimeClockGetTimestamp{ response: map_ptr(proc, args) }
            },

            // Teletype
            0x5401 =>
            {
                IOControlCommand::TeletypeGetSettings{ response: map_ptr(proc, args) }
            },
            0x5402 =>
            {
                IOControlCommand::TeletypeSetSettingsNoWait{ response: map_ptr(proc, args) }
            }
            0x5403 =>
            {
                IOControlCommand::TeletypeSetSettingsDrain{ response: map_ptr(proc, args) }
            }
            0x5404 =>
            {
                IOControlCommand::TeletypeSetSettingsFlush{ response: map_ptr(proc, args) }
            }
            0x540F =>
            {
                IOControlCommand::TeletypeGetProcessGroup{ response: map_ptr(proc, args) }
            }
            0x5410 =>
            {
                IOControlCommand::TeletypeSetProcessGroup{ response: map_ptr(proc, args) }
            }

            default =>
                {
                    kwarnln!("Unknown ioctl command 0x{:x} from PID {}", default, proc.pid);
                    return 0;
                }
        };
        
    proc.exec_ioctl(fd, structured_command)
}