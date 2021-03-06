use core::convert::Into;

#[cfg(not(feature = "std"))]
use alloc::{format, string::*};

/// Owned Path Object
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct OwnedPath
{
    path: String
}

impl OwnedPath
{
    /// Allocate a path on the heap
    pub fn new<T: Into<String>>(data: T) -> Self
    {
        Self
        {
            path: data.into(),
        }
    }

    /// Convert the path to an iter
    pub fn iter(&self) -> PathIterator
    {
        PathIterator
        {
            path: &self,
            iter_index: None,
        }
    }

    /// Canonicalize the path given a CWD
    pub fn canonicalize(&mut self, cwd: PathBuffer)
    {
        if !self.path.starts_with("/")
        {
            let sep = if cwd.path.ends_with("/") { "" } else { "/" };
            self.path = format!("{}{}{}", cwd, sep, self.path);
        }
    }

    /// Canonicalize the path given a CWD
    pub fn canonicalized(&mut self, cwd: PathBuffer) -> OwnedPath
    {
        if !self.path.starts_with("/")
        {
            let sep = if cwd.path.ends_with("/") { "" } else { "/" };
            OwnedPath::new(format!("{}{}{}", cwd, sep, self.path))
        }
        else
        {
            self.clone()
        }
    }


    /// Get the path to the parent of the given path, and the name of the final
    /// element of the path
    pub fn split_last(&self) -> (OwnedPath, &str)
    {
        if self.path.len() == 0 || self.path == "/"
        {
            return (OwnedPath::new(""), "");
        }

        // If the path ends with a '/', step back one more character
        let end_index = self.path.len() - 1 - if self.path.ends_with("/") { 1 } else { 0 };

        // Set the starting index to match
        let mut start_index = end_index;

        // Walk backwards to either the last '/' or the start of the string
        while start_index > 1 && self.path.chars().nth(start_index - 1) != Some('/')
        {
            start_index -= 1;
        }

        // Then return the resulting split path
        (OwnedPath::new(&self.path[..start_index]), &self.path[start_index..=end_index])
    }

    /// Get a reference to the inner string
    pub fn as_str(&self) -> &str
    {
        &self.path
    }

    /// Get a mutable reference to the inner string
    pub fn as_mut_str(&mut self) -> &mut String
    {
        &mut self.path
    }
}

impl core::fmt::Display for OwnedPath
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result
    {
        write!(f, "{}", self.path)
    }
}

/// Path Buffer Object, aliases a reference to `OwnedPath`
pub type PathBuffer<'a> = &'a OwnedPath;

/// Path Iterator
///
/// Iterates over the directory names in a path, for example, converting the
/// path `/usr/bin/ls` to an iterator will yield `&str`'s with the same lifetime
/// as the reference to the path, in the following order: `{"usr", "bin", "ls"}`
pub struct PathIterator<'a>
{
    path: PathBuffer<'a>,
    iter_index: Option<usize>
}

impl<'a> core::iter::Iterator for PathIterator<'a>
{
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item>
    {
        // Initialize the iterator to start at the begining of the path
        if self.iter_index.is_none()
        {
            self.iter_index = Some(0);
        }

        if let Some(mut start_index) = self.iter_index
        {
            // If the starting index places us at a '/', step one character
            // forwards
            if self.path.path.chars().nth(start_index) == Some('/')
            {
                start_index += 1;
            }

            // Ensure we are still within the size of the path
            if start_index < self.path.path.len()
            {
                let mut end_index = start_index + 1;

                while end_index < self.path.path.len() && self.path.path.chars().nth(end_index) != Some('/')
                {
                    end_index += 1;
                }

                self.iter_index = Some(end_index);

                // If we ran past the end of the path, get the slice to the end
                // of the path
                if end_index >= self.path.path.len()
                {
                    Some(&self.path.path[start_index..])
                }
                // Otherwise, get the slice to before the '/'
                else
                {
                    Some(&self.path.path[start_index..end_index])
                }
            }
            // If not, return None, as we are done iterating
            else
            {
                None 
            }
        }
        else
        {
            unreachable!()
        }
    }
}