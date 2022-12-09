use std::{
    cell::RefCell,
    error::Error,
    io::{self, BufRead, BufReader},
    rc::Rc,
};

trait Entry {
    fn name(&self) -> &str;
    fn size(&self) -> usize;
}

struct File {
    name: String,
    size: usize,
}

impl File {
    fn new(name: &str, size: usize) -> File {
        File {
            name: name.to_string(),
            size: size,
        }
    }
}

impl Entry for File {
    fn name(&self) -> &str {
        &self.name
    }

    fn size(&self) -> usize {
        self.size
    }
}

struct Directory {
    name: String,
    child_dirs: Vec<Rc<RefCell<Directory>>>,
    child_files: Vec<Rc<File>>,
}

impl Directory {
    fn new(name: &str) -> Rc<RefCell<Directory>> {
        Rc::new(RefCell::new(Directory {
            name: name.to_string(),
            child_dirs: Vec::new(),
            child_files: Vec::new(),
        }))
    }

    fn get_or_create_child_directory_by_name(
        &mut self,
        name: &str,
    ) -> Result<Rc<RefCell<Directory>>, Box<dyn Error>> {
        if let Some(existing) = self
            .child_dirs
            .iter()
            .find(|child| child.borrow().name() == name)
        {
            println!("found existing dir {} in {}", name, self.name);
            Ok(existing.clone())
        } else {
            println!("adding dir {} to {}", name, self.name);
            let result = Directory::new(name);
            self.child_dirs.push(result.clone());
            Ok(result)
        }
    }

    fn get_or_create_child_file_by_name(
        &mut self,
        name: &str,
        size: usize,
    ) -> Result<Rc<File>, Box<dyn Error>> {
        if let Some(existing) = self.child_files.iter().find(|child| child.name() == name) {
            println!("found existing file {} in {}", name, self.name);
            Ok(existing.clone())
        } else {
            println!("adding file {} to {}", name, self.name);
            let result = Rc::new(File::new(name, size));
            self.child_files.push(result.clone());
            Ok(result)
        }
    }

    fn get_or_create_child_directory_by_path<'a, I>(
        d: Rc<RefCell<Directory>>,
        path: I,
    ) -> Result<Rc<RefCell<Directory>>, Box<dyn Error>>
    where
        I: Iterator<Item = &'a str>,
    {
        let mut result = d;
        for component in path {
            result = result
                .clone()
                .borrow_mut()
                .get_or_create_child_directory_by_name(component)?;
        }
        Ok(result)
    }
}

impl Entry for Directory {
    fn name(&self) -> &str {
        &self.name
    }

    fn size(&self) -> usize {
        self.child_dirs
            .iter()
            .map(|child| child.borrow().size())
            .sum::<usize>()
            + self
                .child_files
                .iter()
                .map(|child| child.size())
                .sum::<usize>()
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut r = BufReader::new(io::stdin());
    do_it(&mut r)?;
    Ok(())
}

fn do_it(r: &mut impl std::io::Read) -> Result<Rc<RefCell<Directory>>, Box<dyn Error>> {
    let cd_regex = regex::Regex::new(r"^\$ cd (.+)$")?;
    let ls_regex = regex::Regex::new(r"^\$ ls$")?;
    let dir_regex = regex::Regex::new(r"^dir (.+)$")?;
    let file_regex = regex::Regex::new(r"([0-9]+) (.+)")?;

    let mut current_working_directory = Vec::<String>::new();
    let root_directory = Directory::new("/");

    for line in BufReader::new(r).lines() {
        match line {
            Ok(line) => {
                if let Some(captures) = cd_regex.captures(&line) {
                    let name = &captures[1];
                    match name {
                        "/" => {
                            println!("cmd is go to root");
                            current_working_directory.clear();
                        }
                        ".." => {
                            println!("cmd is go up one");
                            current_working_directory.pop();
                        }
                        _ => {
                            println!("cmd is go to dir = {}", name);
                            current_working_directory.push(name.clone().to_string());
                            Directory::get_or_create_child_directory_by_path(
                                root_directory.clone(),
                                current_working_directory.iter().map(|s| s.as_str()),
                            )?;
                        }
                    };
                    println!("working dir is now {:?}", current_working_directory);
                    Ok(())
                } else if ls_regex.is_match(&line) {
                    println!("cmd is ls");
                    Ok(())
                } else if let Some(captures) = dir_regex.captures(&line) {
                    let name = &captures[1];
                    println!("line is dir = {}", name);
                    Ok(())
                } else if let Some(captures) = file_regex.captures(&line) {
                    let size = captures[1].parse::<usize>()?;
                    let name = &captures[2];
                    let file = Directory::get_or_create_child_directory_by_path(
                        root_directory.clone(),
                        current_working_directory.iter().map(|s| s.as_str()),
                    )?
                    .borrow_mut()
                    .get_or_create_child_file_by_name(name, size)?;
                    println!(
                        "line is file name = {}, size = {}",
                        file.name(),
                        file.size()
                    );
                    Ok(())
                } else {
                    Err(format!("unrecognized line: {}", line))
                }
            }
            Err(e) => Err(e.to_string()),
        }?
    }
    println!("");

    fn pretty_print(d: Rc<RefCell<Directory>>, indent: i32) {
        let d = d.borrow();
        let indent_str = (0..indent).map(|_| "  ").collect::<Vec<&str>>().join("");
        println!("{}- {} (dir)", indent_str, d.name());
        for child in d.child_dirs.iter() {
            pretty_print(child.clone(), indent + 1);
        }
        for child in d.child_files.iter() {
            println!(
                "{}  - {} (file, size={})",
                indent_str,
                child.name(),
                child.size()
            );
        }
    }
    pretty_print(root_directory.clone(), 0);
    println!("");

    const FILESYSTEM_CAPACITY: usize = 70000000;
    const NEEDED_SPACE: usize = 30000000;

    fn find_sizes(
        d: Rc<RefCell<Directory>>,
        root: Rc<RefCell<Directory>>,
        result: &mut Option<Rc<RefCell<Directory>>>,
    ) {
        let total_size_if_deleted = root.borrow().size() - d.borrow().size();
        let remaining_capacity_if_deleted = FILESYSTEM_CAPACITY - total_size_if_deleted;
        if remaining_capacity_if_deleted >= NEEDED_SPACE {
            println!("deleting {} would do it", d.borrow().name());
            if let Some(r) = result {
                if d.borrow().size() < r.borrow().size() {
                    println!(
                        "this one {} is smaller than previous best {}, keeping this one",
                        d.borrow().name(),
                        r.borrow().name()
                    );
                    *result = Some(d.clone());
                } else {
                    println!(
                        "the previous best {} is smaller than this one {}, keeping the previous best",
                        r.borrow().name(),
                        d.borrow().name()
                    );
                }
            } else {
                println!("no best result yet, keeping this one");
                *result = Some(d.clone());
            }
        }
        for child in d.borrow().child_dirs.iter() {
            find_sizes(child.clone(), root.clone(), result);
        }
    }
    let mut best = None;
    find_sizes(root_directory.clone(), root_directory.clone(), &mut best);
    let best = best.ok_or("no best result found")?;
    println!(
        "best directory found = {}, size = {}",
        best.borrow().name(),
        best.borrow().size()
    );
    println!("");

    Ok(best.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample() -> Result<(), Box<dyn Error>> {
        let result = do_it(
            &mut r"$ cd /
$ ls
dir a
14848514 b.txt
8504156 c.dat
dir d
$ cd a
$ ls
dir e
29116 f
2557 g
62596 h.lst
$ cd e
$ ls
584 i
$ cd ..
$ cd ..
$ cd d
$ ls
4060174 j
8033020 d.log
5626152 d.ext
7214296 k"
                .as_bytes(),
        )?;
        assert_eq!("d", result.borrow().name());
        Ok(())
    }
}
