use ac_ffmpeg::codec::audio::{AudioDecoder, AudioFrameMut, AudioResampler, ChannelLayout};
use ac_ffmpeg::codec::Decoder;
use ac_ffmpeg::format::demuxer::{Demuxer, DemuxerWithStreamInfo};
use ac_ffmpeg::format::io::IO;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use std::io::Cursor;

#[pyfunction]
pub fn resample<'a>(py: Python<'a>, data: &'a [u8]) -> PyResult<()> {
    let mut data = Cursor::new(data);
    let mut io = IO::from_seekable_read_stream(data.clone());
    let mut demuxer: DemuxerWithStreamInfo<Cursor<&[u8]>> = Demuxer::builder()
        .build(io)
        .expect("Failed to build demuxer")
        .find_stream_info(None)
        .map_err(|(_, err)| err)
        .unwrap();
    let (index, (stream, _)) = demuxer
        .streams()
        .iter()
        .map(|stream| (stream, stream.codec_parameters()))
        .enumerate()
        .find(|(_, (_, params))| params.is_audio_codec())
        .unwrap();
    let codec = stream.codec_parameters();
    let codec = codec.as_audio_codec_parameters().unwrap();
    let mut resampler = AudioResampler::builder()
        .source_channel_layout(codec.channel_layout().to_owned())
        .source_sample_format(codec.sample_format())
        .source_sample_rate(codec.sample_rate())
        .target_channel_layout(ChannelLayout::from_channels(1).unwrap())
        .target_sample_format(codec.sample_format())
        .target_sample_rate(36000)
        .build()
        .unwrap();
    let mut decoder = AudioDecoder::from_stream(stream).unwrap().build().unwrap();
    while let Ok(Some(packet)) = demuxer.take() {
        println!("packet: {:?}", packet.stream_index());
        if packet.stream_index() != index {
            continue;
        }
        decoder.push(packet).unwrap();
        while let Some(frame) = decoder.take().unwrap() {
            println!("frame: {:?}", frame.pts().as_f32().unwrap_or(0f32));
            resampler.push(frame).unwrap();
        }
    }
    let mut result = Vec::new();
    while let Some(frame) = resampler.take().unwrap() {
        for plane in frame.planes().iter() {
            result.push(plane.data().clone());
        }
    }
    Ok(())
}
