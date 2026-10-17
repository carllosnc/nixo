use std::time::Duration;

use nixo_core::{MediaBackend, PlayerCommand, SubtitleHealth, SubtitleStyle};

use super::MpvBackend;

#[test]
fn test_subtitle_font_and_weight_change() {
    let Ok(mut backend) = MpvBackend::try_new() else {
        return;
    };

    let ass_path = std::env::temp_dir().join(format!("nixo_font_test_{}.ass", std::process::id()));
    let ass_content = "[Script Info]\nScriptType: v4.00+\nPlayResX: 1920\nPlayResY: 1080\n\n[V4+ Styles]\nFormat: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\nStyle: Default,Arial,58,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,-1,0,0,0,100,100,0,0,1,2,1,2,20,20,40,1\nStyle: MainDialog,Trebuchet MS,58,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,-1,0,0,0,100,100,0,0,1,2,1,2,20,20,40,1\n\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\nDialogue: 0,0:00:00.00,0:00:10.00,MainDialog,,0,0,0,,{\\fnArial\\b1\\c&HFFFFFF&}That belief is dangerous.\n";
    std::fs::write(&ass_path, ass_content).expect("write temp ass");

    backend
        .load_uri(
            "av://lavfi:color=c=black:s=1280x720:r=30:d=5",
            std::path::PathBuf::from("black"),
        )
        .expect("load black video");
    std::thread::sleep(Duration::from_millis(150));
    backend
        .execute(PlayerCommand::LoadSubtitleFile(ass_path.clone()))
        .expect("load ass subtitle");
    backend.execute(PlayerCommand::Pause).expect("pause");
    std::thread::sleep(Duration::from_millis(150));

    let mut render_style = |font: &str, bold: bool| -> Vec<u8> {
        let st = SubtitleStyle {
            font_family: font.to_string(),
            bold,
            ..Default::default()
        };
        backend
            .execute(PlayerCommand::SetSubtitleStyle(st))
            .expect("set style");
        let mut last = None;
        for _ in 0..8 {
            if let Some(f) = backend.poll_video_frame((1280, 720)) {
                last = Some(f.rgba);
            }
            std::thread::sleep(Duration::from_millis(25));
        }
        last.expect("rendered frame")
    };

    let f_inter_reg = render_style("Inter", false);
    let f_inter_bold = render_style("Inter", true);
    let f_oswald_reg = render_style("Oswald", false);

    let _ = std::fs::remove_file(&ass_path);

    assert_ne!(
        f_inter_reg, f_inter_bold,
        "Normal e Bold devem gerar quadros distintos"
    );
    assert_ne!(
        f_inter_reg, f_oswald_reg,
        "Inter e Oswald devem gerar quadros distintos mesmo em ASS com tags inline"
    );
    assert_eq!(backend.status().subtitle_health, SubtitleHealth::Ok);
}
