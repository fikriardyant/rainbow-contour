use crate::marching_squares::IsolineSegment;

pub fn export_isolines_to_dxf(segments: &[IsolineSegment]) -> String {
    let mut dxf = String::new();
    dxf.push_str("0\nSECTION\n2\nENTITIES\n");

    for seg in segments {
        dxf.push_str("0\nLINE\n8\nRAINBOW_CONTOUR\n");
        dxf.push_str(&format!("62\n{}\n", seg.color_aci));
        dxf.push_str(&format!("10\n{}\n20\n{}\n30\n{}\n", seg.p0.x, seg.p0.y, seg.p0.z));
        dxf.push_str(&format!("11\n{}\n21\n{}\n31\n{}\n", seg.p1.x, seg.p1.y, seg.p1.z));
    }

    dxf.push_str("0\nENDSEC\n0\nEOF\n");
    dxf
}
