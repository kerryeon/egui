use criterion::{criterion_group, criterion_main, Criterion};

pub fn criterion_benchmark(c: &mut Criterion) {
    let raw_input = egui::RawInput {
        screen_size: egui::vec2(1280.0, 1024.0),
        ..Default::default()
    };

    {
        let mut ctx = egui::Context::new();
        let mut demo_windows = egui::demos::DemoWindows::default();

        c.bench_function("demo_windows_minimal", |b| {
            b.iter(|| {
                ctx.begin_frame(raw_input.clone());
                demo_windows.ui(&ctx, &Default::default(), &mut None);
                ctx.end_frame()
            })
        });
    }

    {
        let mut ctx = egui::Context::new();
        ctx.memory().all_collpasing_are_open = true; // expand the demo window with everything
        let mut demo_windows = egui::demos::DemoWindows::default();

        c.bench_function("demo_windows_full", |b| {
            b.iter(|| {
                ctx.begin_frame(raw_input.clone());
                demo_windows.ui(&ctx, &Default::default(), &mut None);
                ctx.end_frame()
            })
        });
    }

    {
        let mut ctx = egui::Context::new();
        ctx.memory().all_collpasing_are_open = true; // expand the demo window with everything
        let mut demo_windows = egui::demos::DemoWindows::default();
        ctx.begin_frame(raw_input.clone());
        demo_windows.ui(&ctx, &Default::default(), &mut None);
        let (_, paint_commands) = ctx.end_frame();

        c.bench_function("tesselate", |b| {
            b.iter(|| ctx.tesselate(paint_commands.clone()))
        });
    }

    {
        let mut ctx = egui::Context::new();
        ctx.begin_frame(raw_input);
        egui::CentralPanel::default().show(&ctx, |ui| {
            c.bench_function("label", |b| {
                b.iter(|| {
                    ui.label(egui::demos::LOREM_IPSUM_LONG);
                })
            });
        });
        let _ = ctx.end_frame();
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
