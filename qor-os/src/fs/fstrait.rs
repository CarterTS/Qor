//! Filesystem Traits

use crate::*;

use super::structures::*;

/// Generic Filesystem Trait
pub trait Filesystem
{
    /// Initialize the filesystem on the current disk
    fn init(&mut self) -> FilesystemResult<()>;

    /// Sync the filesystem with the current disk
    fn sync(&mut self) -> FilesystemResult<()>;

    /// Set the mount_id of the filesystem
    fn set_mount_id(&mut self, mount_id: usize, vfs: &'static mut crate::fs::vfs::FilesystemInterface);

    /// Get the index of the root directory of the filesystem
    fn get_root_index(&mut self) -> FilesystemResult<FilesystemIndex>;

    /// Convert a path to an inode
    fn path_to_inode(&mut self, path: &str) -> FilesystemResult<FilesystemIndex>;

    /// Convert an inode to a path
    fn inode_to_path(&mut self, inode: FilesystemIndex) -> FilesystemResult<&str>;

    /// Get the directory entries for the given inode
    fn get_dir_entries(&mut self, inode: FilesystemIndex) -> FilesystemResult<Vec<DirectoryEntry>>;

    /// Create a file in the directory at the given inode
    fn create_file(&mut self, inode: FilesystemIndex, name: String) -> FilesystemResult<FilesystemIndex>;

    /// Create a directory in the directory at the given inode
    fn create_directory(&mut self, inode: FilesystemIndex, name: String) -> FilesystemResult<FilesystemIndex>;

    /// Remove an inode at the given index from the given directory
    fn remove_inode(&mut self, inode: FilesystemIndex, directory: FilesystemIndex) -> FilesystemResult<()>;

    /// Read the data stored in an inode
    fn read_inode(&mut self, inode: FilesystemIndex) -> FilesystemResult<Vec<u8>>;

    /// Write data to an inode
    fn write_inode(&mut self, inode: FilesystemIndex, data: &[u8]) -> FilesystemResult<()>;

    /// Mount a filesystem at the given inode
    fn mount_fs_at(&mut self, inode: FilesystemIndex, root: FilesystemIndex, name: String) -> FilesystemResult<()>;
}