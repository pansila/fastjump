# fastjump
Jump to any directory fast and smart in the shell.

It's a port of the wildly popular helper application [autojump](https://github.com/wting/autojump) to Rust.

# Why porting
1. `autojump` is no long maintained and there are a few bugs preventing people from enjoying it.
2. The author is really tired of `autojump` breakage inside Python `virtualenvs`.
3. Rust is simply awesome for CLI applications, with its performance and (code) slickness!
4. There is another Rust port of [autojump](https://github.com/xen0n/autojump-rs), which targets a drop-in replacement of `autojump`. This project instead aims at faster CLI invoking and responding (by injecting into shell).

# Install
We have prebuilt binaries available for a while now, thanks to the [trust](https://github.com/japaric/trust) project!

# Features
1. By re-writing in Rust, `fastjump` is more light-weight than python version of `autojump`. As the program itself is very short-running, the overhead of setting up and tearing down a whole Python VM could be overwhelming, especially on less capable hardware.

   ```
   (some benchmarks need to go here)
   ```
2. Using `serde` with `bincode` to provide a bleeding fast serialization/deserialization to database file.
3. Injecting into the shell to stay in the shell's memory space to speed up command invoking much further (WIP).

# Compatibility
1. All of the command line flags and arguments of `autojump` are implemented, and behave exactly like the original. All other shell features like tab completion should work too. (Except jc and jco; see below.)

2. Since we use `bincode` to support database, it's not a drop-in replacement of `autojump`. However we provide a tool to import `autojump`'s command history to re-use your work history.

# Reporting bugs

For any questions or issues please visit:

    https://github.com/pansila/fastjump/issues

# Contribution
Welcome PRs.
