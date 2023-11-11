pub mod metadata;
pub mod types;
pub mod utils;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let src_unpack = r"C:\Program Files (x86)\Steam\steamapps\common\Exanima\Textures.rpk";
    let dest_unpack = r"C:\Program Files (x86)\Steam\steamapps\common\Exanima\unpacked\Textures";
    types::rpk::RPK::unpack(src_unpack, dest_unpack)?;

    // let src_unpack_all = r"C:\Program Files (x86)\Steam\steamapps\common\Exanima\unpacked\Resource";
    // let dest_unpack_all = r"C:\Program Files (x86)\Steam\steamapps\common\Exanima\unpacked";
    // utils::unpack_all(src_unpack_all, dest_unpack_all)?;

    // let src_pack = r"C:\Program Files (x86)\Steam\steamapps\common\Exanima\unpacked\Characters";
    // let dest_pack = r"C:\Program Files (x86)\Steam\steamapps\common\Exanima\packed";
    // types::rpk::RPK::pack(src_pack, dest_pack)?;

    // let src_pack_all = r"C:\Program Files (x86)\Steam\steamapps\common\Exanima\unpacked";
    // let dest_pack_all = r"C:\Program Files (x86)\Steam\steamapps\common\Exanima\packed";
    // utils::pack_all(src_pack_all, dest_pack_all)?;

    Ok(())
}
