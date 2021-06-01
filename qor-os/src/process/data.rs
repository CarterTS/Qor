use alloc::collections::BTreeMap;

use crate::*;

/// Process Data
pub struct ProcessData
{
    pub stack_size: usize, // Stack size in pages
    pub mem_ptr: *mut u8,
    pub mem_size: usize, // Size of the memory allocated in pages,
    pub descriptors: BTreeMap<usize, Box<dyn super::descriptor::FileDescriptor>>,
    pub children: Vec<u16>,
    pub parent_pid: u16,
    pub cwd: String,
}

impl ProcessData
{
    /// Initialize a fresh process data
    /// Safety: The mem_ptr must be valid or zero
    pub unsafe fn new(stack_size: usize, mem_ptr: usize, mem_size: usize) -> Self
    {
        let mut descriptors: BTreeMap<usize, Box<dyn super::descriptor::FileDescriptor>> = BTreeMap::new();

        descriptors.insert(0, Box::new(super::descriptor::NullDescriptor{}));
        descriptors.insert(1, Box::new(super::descriptor::NullDescriptor{}));
        descriptors.insert(2, Box::new(super::descriptor::NullDescriptor{}));

        Self
        {
            stack_size,
            mem_ptr: mem_ptr as *mut u8,
            mem_size,
            descriptors,
            children: Vec::new(),
            parent_pid: 0,
            cwd: String::from("/bin/")
        }
    }

    /// Connect the process to stdin, stderr, and stdout
    pub fn connect_to_term(&mut self)
    {
        self.descriptors.insert(0, Box::new(super::descriptor::UARTIn{}));
        self.descriptors.insert(1, Box::new(super::descriptor::UARTOut{}));
        self.descriptors.insert(2, Box::new(super::descriptor::UARTError{}));
    }

    /// Register a child process
    pub fn register_child(&mut self, child_pid: u16)
    {
        self.children.push(child_pid);
    }

    /// Set the parent PID
    pub fn set_parent(&mut self, parent: u16)
    {
        self.parent_pid = parent;
    }
}