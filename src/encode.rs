use crate::{CompoundTag, Tag};
use byteorder::{BigEndian, WriteBytesExt};
use flate2::write::{GzEncoder, ZlibEncoder};
use std::io::{Error, Write};

/// Write a compound tag to writer using gzip compression.
pub fn write_gzip_compound_tag<W: Write>(
    writer: &mut W,
    compound_tag: CompoundTag,
) -> Result<(), Error> {
    write_compound_tag(
        &mut GzEncoder::new(writer, Default::default()),
        compound_tag,
    )
}

/// Write a compound tag to writer using zlib compression.
pub fn write_zlib_compound_tag<W: Write>(
    writer: &mut W,
    compound_tag: CompoundTag,
) -> Result<(), Error> {
    write_compound_tag(
        &mut ZlibEncoder::new(writer, Default::default()),
        compound_tag,
    )
}

/// Write a compound tag to writer.
///
/// # Example
/// ```
/// use nbt::encode::write_compound_tag;
/// use nbt::CompoundTag;
///
/// let mut server = CompoundTag::new();
///
/// server.insert_str("ip", "localhost:25565");
/// server.insert_str("name", "Minecraft Server");
/// server.insert_bool("hideAddress", true);
///
/// let mut servers = Vec::new();
/// servers.push(server);
///
/// let mut root_tag = CompoundTag::new();
/// root_tag.insert_compound_tag_vec("servers", servers);
///
/// let mut vec = Vec::new();
/// write_compound_tag(&mut vec, root_tag).unwrap();
/// ```
pub fn write_compound_tag<W: Write>(
    writer: &mut W,
    compound_tag: CompoundTag,
) -> Result<(), Error> {
    let name = compound_tag.name.as_deref().unwrap_or("");
    let tag = Tag::Compound(compound_tag.clone());

    writer.write_u8(tag.type_id())?;
    write_string(writer, name)?;

    write_tag(writer, tag)
}

fn write_tag<W: Write>(writer: &mut W, tag: Tag) -> Result<(), Error> {
    match tag {
        Tag::Byte(value) => writer.write_i8(value)?,
        Tag::Short(value) => writer.write_i16::<BigEndian>(value)?,
        Tag::Int(value) => writer.write_i32::<BigEndian>(value)?,
        Tag::Long(value) => writer.write_i64::<BigEndian>(value)?,
        Tag::Float(value) => writer.write_f32::<BigEndian>(value)?,
        Tag::Double(value) => writer.write_f64::<BigEndian>(value)?,
        Tag::ByteArray(value) => {
            writer.write_u32::<BigEndian>(value.len() as u32)?;

            for v in value {
                writer.write_i8(v)?;
            }
        }
        Tag::String(value) => write_string(writer, &value)?,
        Tag::List(value) => {
            if value.len() > 0 {
                writer.write_u8(value[0].type_id())?;
            } else {
                // Empty list type.
                writer.write_u8(0)?;
            }

            writer.write_u32::<BigEndian>(value.len() as u32)?;

            for tag in value {
                write_tag(writer, tag)?;
            }
        }
        Tag::Compound(value) => {
            for (name, tag) in value.tags {
                writer.write_u8(tag.type_id())?;
                write_string(writer, &name)?;
                write_tag(writer, tag)?;
            }

            // To mark compound tag end.
            writer.write_u8(0)?;
        }
        Tag::IntArray(value) => {
            writer.write_u32::<BigEndian>(value.len() as u32)?;

            for v in value {
                writer.write_i32::<BigEndian>(v)?;
            }
        }
        Tag::LongArray(value) => {
            writer.write_u32::<BigEndian>(value.len() as u32)?;

            for v in value {
                writer.write_i64::<BigEndian>(v)?;
            }
        }
    }

    Ok(())
}

fn write_string<W: Write>(writer: &mut W, value: &str) -> Result<(), Error> {
    writer.write_u16::<BigEndian>(value.len() as u16)?;
    writer.write(value.as_bytes())?;

    Ok(())
}

#[test]
fn test_hello_world_write() {
    let mut hello_world = CompoundTag::named("hello world");
    hello_world.insert_str("name", "Bananrama");

    let mut vec = Vec::new();
    write_compound_tag(&mut vec, hello_world).unwrap();

    assert_eq!(
        vec,
        include_bytes!("../test/binary/hello_world.dat").to_vec()
    );
}

#[test]
fn test_servers_write() {
    let mut server = CompoundTag::new();

    server.insert_str("ip", "localhost:25565");
    server.insert_str("name", "Minecraft Server");
    server.insert_bool("hideAddress", true);

    let mut servers = Vec::new();
    servers.push(server);

    let mut root_tag = CompoundTag::new();
    root_tag.insert_compound_tag_vec("servers", servers);

    let mut vec = Vec::new();
    write_compound_tag(&mut vec, root_tag).unwrap();

    assert_eq!(vec, include_bytes!("../test/binary/servers.dat").to_vec());
}
