//! Filesystem Traits

use crate::*;

use super::structures::*;

use libutils::paths::PathBuffer;

use super::ioctl::*;

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
    fn path_to_inode(&mut self, path: PathBuffer) -> FilesystemResult<FilesystemIndex>;

    /// Convert an inode to a path
    fn inode_to_path(&mut self, inode: FilesystemIndex) -> FilesystemResult<PathBuffer>;

    /// Get the directory entries in the directory at the given inode
    fn get_dir_entries(&mut self, inode: FilesystemIndex) -> FilesystemResult<Vec<DirectoryEntry>>;

    /// Get the directory entry for the given inode
    fn get_stat(&mut self, inode: FilesystemIndex) -> FilesystemResult<FileStat>;

    /// Create a file in the directory at the given inode
    fn create_file(&mut self, inode: FilesystemIndex, name: String) -> FilesystemResult<FilesystemIndex>;

    /// Create a directory in the directory at the given inode
    fn create_directory(&mut self, inode: FilesystemIndex, name: String) -> FilesystemResult<FilesystemIndex>;

    /// Remove an inode at the given index from the given directory
    fn remove_inode(&mut self, inode: FilesystemIndex) -> FilesystemResult<()>;

    /// Remove a directory entry from the directory at the given inode
    fn remove_dir_entry(&mut self, directory_index: FilesystemIndex, name: String) -> FilesystemResult<()>;

    /// Increment the number of links to an inode
    fn increment_links(&mut self, inode: FilesystemIndex) -> FilesystemResult<usize>;

    /// Decrement the number of links to an inode
    fn decrement_links(&mut self, inode: FilesystemIndex) -> FilesystemResult<usize>;

    /// Read the data stored in an inode
    fn read_inode(&mut self, inode: FilesystemIndex) -> FilesystemResult<Vec<u8>>;

    /// Write data to an inode
    fn write_inode(&mut self, inode: FilesystemIndex, data: &[u8]) -> FilesystemResult<()>;

    /// Mount a filesystem at the given inode
    fn mount_fs_at(&mut self, inode: FilesystemIndex, root: FilesystemIndex, name: String) -> FilesystemResult<()>;

    /// Open a filedescriptor for the given inode
    fn open_fd(&mut self, inode: FilesystemIndex, mode: usize) -> FilesystemResult<Box<dyn crate::process::descriptor::FileDescriptor>>;

    /// Execute an ioctl command on an inode
    fn exec_ioctl(&mut self, inode: FilesystemIndex, cmd: IOControlCommand) -> FilesystemResult<usize>;

    /// Assert is not a directory
    fn assert_not_directory(&mut self, inode: FilesystemIndex) -> FilesystemResult<()>
    {
        if self.get_stat(inode)?.mode & 0x4000 > 0
        {
            Err(FilesystemError::INodeIsDirectory)
        }
        else
        {
            Ok(())
        }
    }

    /// Assert is a directory
    fn assert_directory(&mut self, inode: FilesystemIndex) -> FilesystemResult<()>
    {
        if self.get_stat(inode)?.mode & 0x4000 > 0
        {
            Ok(())
        }
        else
        {
            Err(FilesystemError::INodeIsNotADirectory)
        }
    }

    /// Unlink an inode
    fn unlink_inode(&mut self, inode: FilesystemIndex, directory: FilesystemIndex, name: String) -> FilesystemResult<()>
    {
        self.assert_not_directory(inode)?;
        self.assert_directory(directory)?;

        if self.decrement_links(inode)? == 0
        {
            self.remove_dir_entry(directory, name)?;
            self.remove_inode(inode)?;
        }

        Ok(())
    }

    /// Unlink an inode
    fn remove_directory(&mut self, inode: FilesystemIndex, parent: FilesystemIndex, name: String) -> FilesystemResult<()>
    {
        self.assert_directory(inode)?;
        self.assert_directory(parent)?;

        // Make sure the directory is empty
        for ent in self.get_dir_entries(inode)?
        {
            if ent.name != "." && ent.name != ".."
            {
                return Err(FilesystemError::DirectoryNotEmpty);
            }
        }

        // Remove the directory entry for the directory in its parent and remove the inode for the directory
        self.remove_dir_entry(parent, name)?;
        self.remove_inode(inode)?;

        Ok(())
    }
}