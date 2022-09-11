fn main() {
    protobuf_codegen::Codegen::new()
        .cargo_out_dir("protos")
        .protoc()
        .include("src/ProPresenter7-Proto/proto")
        .input("src/ProPresenter7-Proto/proto/propresenter.proto")
        .input("src/ProPresenter7-Proto/proto/keymapping.proto")
        .input("src/ProPresenter7-Proto/proto/basicTypes.proto")
        .input("src/ProPresenter7-Proto/proto/playlist.proto")
        .input("src/ProPresenter7-Proto/proto/hotKey.proto")
        .input("src/ProPresenter7-Proto/proto/cue.proto")
        .input("src/ProPresenter7-Proto/proto/action.proto")
        .input("src/ProPresenter7-Proto/proto/layers.proto")
        .input("src/ProPresenter7-Proto/proto/effects.proto")
        .input("src/ProPresenter7-Proto/proto/graphicsData.proto")
        .input("src/ProPresenter7-Proto/proto/template.proto")
        .input("src/ProPresenter7-Proto/proto/presentationSlide.proto")
        .input("src/ProPresenter7-Proto/proto/background.proto")
        .input("src/ProPresenter7-Proto/proto/timers.proto")
        .input("src/ProPresenter7-Proto/proto/stage.proto")
        .input("src/ProPresenter7-Proto/proto/messages.proto")
        .input("src/ProPresenter7-Proto/proto/digitalAudio.proto")
        .input("src/ProPresenter7-Proto/proto/planningCenter.proto")
        .input("src/ProPresenter7-Proto/proto/slide.proto")
        .input("src/ProPresenter7-Proto/proto/alignmentGuide.proto")
        .input("src/ProPresenter7-Proto/proto/input.proto")
        .input("src/ProPresenter7-Proto/proto/propSlide.proto")
        .input("src/ProPresenter7-Proto/proto/timestamp.proto")
        .input("src/ProPresenter7-Proto/proto/presentation.proto")
        .input("src/ProPresenter7-Proto/proto/groups.proto")
        .run_from_script();
}