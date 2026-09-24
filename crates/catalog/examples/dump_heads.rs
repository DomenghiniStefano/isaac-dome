//! Spike: prints the crops of coop menu.anm2's Main layer and a PowerShell command
//! to slice the sheet, so the heads can be looked at and the map written.

fn main() {
    let Some(packed) = test_support::packed_dir() else {
        return;
    };
    let rs = unpack::ResourceSet::open(&packed);
    let Some(anm2) = rs.read("gfx/ui/coop menu.anm2") else {
        println!("anm2 missing: needs samples/packed");
        return;
    };
    let mut d = Vec::new();
    let frames = catalog::for_tests::parse_heads(&anm2, &mut d);
    println!("{} frames in the Main layer", frames.len());
    let out = test_support::samples_dir().join("sprites").join("heads");
    println!(
        "# in PowerShell, after `cargo run -p unpack --example estrai -- \"gfx/ui/coop menu.png\"`:"
    );
    println!(
        "Add-Type -AssemblyName System.Drawing; New-Item -ItemType Directory -Force '{}' | Out-Null",
        out.display()
    );
    println!("$src = [System.Drawing.Image]::FromFile('C:\\Projects\\isaac-dome\\samples\\sprites\\coop_menu.png')");
    for (i, f) in frames.iter().enumerate() {
        let Some(r) = f else {
            println!("# frame {i}: no crop");
            continue;
        };
        println!(
            "$b = New-Object System.Drawing.Bitmap {}, {}; $g = [System.Drawing.Graphics]::FromImage($b); $g.DrawImage($src, (New-Object System.Drawing.Rectangle 0,0,{},{}), (New-Object System.Drawing.Rectangle {},{},{},{}), 'Pixel'); $b.Save('{}\\frame_{:02}.png'); $g.Dispose(); $b.Dispose()",
            r.w * 4, r.h * 4, r.w * 4, r.h * 4, r.x, r.y, r.w, r.h, out.display(), i
        );
    }
}
