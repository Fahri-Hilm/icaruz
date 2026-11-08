# Wine check: Windows target tanpa root

Validasi kode Rust untuk Windows dari Linux pakai zig + Wine.
Hasil terakhir: 38/38 unit test PASS sebagai `.exe` di Wine 11.

## Syarat sekali saja
1. `rustup target add x86_64-pc-windows-gnu`
2. Download zig linux x86_64 (55 MB), extract ke `~/.local/zig`.
3. Wrapper di `~/.local/bin`:
   - `zig-cc-windows`: `zig cc -target x86_64-windows-gnu`, buang flag
     `--target=x86_64-pc-windows-gnu -lmsvcrt -lpthread -l:libpthread.a
     -lmingwex -lmingw32 -lgcc -lgcc_eh` (zig sediakan sendiri).
   - `zig-ar-windows`: `zig ar`.
   - `x86_64-w64-mingw32-dlltool`: `zig dlltool` (rustc butuh untuk raw-dylib).
4. Bangun libunwind sekali: `zig build-lib libunwind/src/libunwind.cpp`
   `-target x86_64-windows-gnu -lc -D_LIBUNWIND_IS_NATIVE_ONLY`
   ke `/tmp/opencode/libunwind.a` (test harness butuh `_Unwind_DeleteException`).

## Jalankan
```
export PATH="$HOME/.local/bin:$PATH"
export CC_x86_64_pc_windows_gnu="$HOME/.local/bin/zig-cc-windows"
export AR_x86_64_pc_windows_gnu="$HOME/.local/bin/zig-ar-windows"
export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUSTFLAGS="-L /tmp/opencode -C link-args=-lunwind"
cargo test -p icaruz-core --target x86_64-pc-windows-gnu
wine target/x86_64-pc-windows-gnu/debug/deps/icaruz_core-*.exe
```

## Batas jujur
- Ini cek logika + kompilasi Windows, bukan full app (Tauri/WebView2 butuh Windows asli).
- TLS core pakai ring, bukan aws-lc (aws-lc butuh dlltool+mingw penuh).
- Gate penuh tetap di GitHub Actions `windows-latest` (file workflow tambah via web).
