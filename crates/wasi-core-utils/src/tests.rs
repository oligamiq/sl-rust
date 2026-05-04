#[cfg(test)]
mod tests {
    use crate::utils;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn test_echo() {
        let args = vec!["echo", "hello", "world"];
        assert!(utils::echo::execute(args).is_ok());
    }

    #[test]
    fn test_pwd() {
        let args: Vec<String> = vec![];
        assert!(utils::pwd::execute(args).is_ok());
    }

    #[test]
    fn test_mkdir_rmdir() {
        let tmp = tempdir().unwrap();
        let dir_path = tmp.path().join("test_dir");
        let dir_str = dir_path.to_str().unwrap();

        // mkdir
        let args = vec!["mkdir", dir_str];
        assert!(utils::mkdir::execute(args).is_ok());
        assert!(dir_path.exists());
        assert!(dir_path.is_dir());

        // rmdir
        let args = vec!["rmdir", dir_str];
        assert!(utils::rmdir::execute(args).is_ok());
        assert!(!dir_path.exists());
    }

    #[test]
    fn test_touch_rm_unlink() {
        let tmp = tempdir().unwrap();
        let file_path = tmp.path().join("test_file");
        let file_str = file_path.to_str().unwrap();

        // touch
        let args = vec!["touch", file_str];
        assert!(utils::touch::execute(args).is_ok());
        assert!(file_path.exists());

        // rm
        let args = vec!["rm", file_str];
        assert!(utils::rm::execute(args).is_ok());
        assert!(!file_path.exists());

        // unlink
        fs::write(&file_path, "test").unwrap();
        let args = vec!["unlink", file_str];
        assert!(utils::unlink::execute(args).is_ok());
        assert!(!file_path.exists());
    }

    #[test]
    fn test_cp_mv() {
        let tmp = tempdir().unwrap();
        let src = tmp.path().join("src");
        let dst = tmp.path().join("dst");
        let src_str = src.to_str().unwrap();
        let dst_str = dst.to_str().unwrap();

        fs::write(&src, "hello").unwrap();

        // cp
        let args = vec!["cp", src_str, dst_str];
        assert!(utils::cp::execute(args).is_ok());
        assert!(dst.exists());
        assert_eq!(fs::read_to_string(&dst).unwrap(), "hello");

        // mv
        let dst2 = tmp.path().join("dst2");
        let dst2_str = dst2.to_str().unwrap();
        let args = vec!["mv", dst_str, dst2_str];
        assert!(utils::mv::execute(args).is_ok());
        assert!(!dst.exists());
        assert!(dst2.exists());
        assert_eq!(fs::read_to_string(&dst2).unwrap(), "hello");
    }

    #[test]
    fn test_cat_tail() {
        let tmp = tempdir().unwrap();
        let file = tmp.path().join("file");
        let file_str = file.to_str().unwrap();
        fs::write(&file, "line1\nline2\nline3\n").unwrap();

        // cat
        let args = vec!["cat", file_str];
        assert!(utils::cat::execute(args).is_ok());

        // tail
        let args = vec!["tail", "-n", "2", file_str];
        assert!(utils::tail::execute(args).is_ok());
    }

    #[test]
    fn test_ls_tree() {
        let tmp = tempdir().unwrap();
        let dir_str = tmp.path().to_str().unwrap();

        // ls
        let args = vec!["ls", dir_str];
        assert!(utils::ls::execute(args).is_ok());

        // tree
        let args = vec!["tree", dir_str];
        assert!(utils::tree::execute(args).is_ok());
    }

    #[test]
    fn test_uname_arch() {
        assert!(utils::uname::execute(vec!["uname", "-a"]).is_ok());
        assert!(utils::arch::execute(vec!["arch"]).is_ok());
    }

    #[test]
    fn test_sleep() {
        let args = vec!["sleep", "0.01"];
        assert!(utils::sleep::execute(args).is_ok());
    }

    #[test]
    fn test_tee() {
        let tmp = tempdir().unwrap();
        let _file = tmp.path().join("tee_out");
        
        // tee - This is hard to test with stdin without a wrapper, 
        // but we can check success with empty input or just check the file creation if we could mock stdin.
        // For now, just check it parses and runs (it will wait for stdin if called normally).
        // Let's skip interactive part or use a file redirect in a real integration test.
    }

    #[test]
    fn test_ln_link() {
        let tmp = tempdir().unwrap();
        let src = tmp.path().join("src");
        let dst = tmp.path().join("dst");
        let src_str = src.to_str().unwrap();
        let dst_str = dst.to_str().unwrap();

        fs::write(&src, "hello").unwrap();

        // link (hard link)
        let args = vec!["link", src_str, dst_str];
        assert!(utils::link::execute(args).is_ok());
        assert!(dst.exists());

        // ln
        let dst2 = tmp.path().join("dst2");
        let dst2_str = dst2.to_str().unwrap();
        let args = vec!["ln", src_str, dst2_str];
        assert!(utils::ln::execute(args).is_ok());
        assert!(dst2.exists());
    }
}
