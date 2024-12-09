fn main() {
    // Link against system FFmpeg libraries
    println!("cargo:rustc-link-search=native=/opt/homebrew/lib");
    println!("cargo:rustc-link-lib=dylib=avcodec");
    println!("cargo:rustc-link-lib=dylib=avformat");
    println!("cargo:rustc-link-lib=dylib=avutil");
    println!("cargo:rustc-link-lib=dylib=swscale");
    
    // Link against required macOS frameworks
    if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-lib=framework=VideoToolbox");
        println!("cargo:rustc-link-lib=framework=CoreMedia");
        println!("cargo:rustc-link-lib=framework=CoreFoundation");
    }
    
    // Rerun build script if FFmpeg libraries change
    println!("cargo:rerun-if-changed=/opt/homebrew/lib/libavcodec.dylib");
    println!("cargo:rerun-if-changed=/opt/homebrew/lib/libavformat.dylib");
    println!("cargo:rerun-if-changed=/opt/homebrew/lib/libavutil.dylib");
    println!("cargo:rerun-if-changed=/opt/homebrew/lib/libswscale.dylib");
} 