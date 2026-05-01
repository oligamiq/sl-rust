use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};

static SMOKE_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Copy)]
pub struct Particle {
    pub x: i32,
    pub y: i32,
    pub pattern: usize,
    pub kind: usize,
}

pub struct Smoke {
    particles: VecDeque<Particle>,
    should_generate: bool,
}

thread_local! {
    static SMOKE: std::cell::RefCell<Smoke> = const {
        std::cell::RefCell::new(Smoke {
            particles: VecDeque::new(),
            should_generate: false,
        })
    };
}

pub fn set_generation_gate(should_gen: bool) {
    SMOKE.with(|s| {
        let mut smoke = s.borrow_mut();
        smoke.should_generate = should_gen;
    });
}

pub fn add_smoke(x: i32, y: i32) {
    SMOKE.with(|s| {
        let mut smoke = s.borrow_mut();
        if smoke.should_generate && smoke.particles.len() < 1000 {
            let counter = SMOKE_COUNTER.fetch_add(1, Ordering::SeqCst);
            let kind = counter % 2;
            smoke.particles.push_back(Particle {
                x,
                y,
                pattern: 0,
                kind,
            });
        }
    });
}

pub fn update_smoke() {
    SMOKE.with(|s| {
        let mut smoke = s.borrow_mut();
        if !smoke.should_generate {
            return;
        }
        for particle in &mut smoke.particles {
            use crate::train::ascii::{SMOKE_DY, SMOKE_DX};
            
            let dy = SMOKE_DY[particle.pattern];
            let dx = SMOKE_DX[particle.pattern];
            
            particle.y -= dy;
            particle.x += dx;
            particle.pattern += 1;
        }
        smoke.particles.retain(|p| p.pattern < 16);
    });
}

pub fn get_smoke_particles() -> Vec<Particle> {
    SMOKE.with(|s| {
        let smoke = s.borrow();
        smoke.particles.iter().copied().collect()
    })
}

pub fn clear_smoke() {
    SMOKE.with(|s| {
        let mut smoke = s.borrow_mut();
        smoke.particles.clear();
    });
}
