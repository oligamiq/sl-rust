use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};

static SMOKE_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone)]
pub struct Particle {
    pub x: i32,
    pub y: i32,
    pub pattern: usize,
    pub kind: usize,
}

pub struct Smoke {
    particles: VecDeque<Particle>,
}

thread_local! {
    static SMOKE: std::cell::RefCell<Smoke> = const {
        std::cell::RefCell::new(Smoke {
            particles: VecDeque::new(),
        })
    };
}

pub fn add_smoke(x: i32, y: i32) {
    SMOKE.with(|s| {
        let mut smoke = s.borrow_mut();
        if smoke.particles.len() < 1000 {
            let counter = SMOKE_COUNTER.fetch_add(1, Ordering::SeqCst);
            let kind = counter % 5;
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
        for particle in &mut smoke.particles {
            particle.x -= 1;
            particle.y -= 1;
            particle.pattern += 1;
        }
        smoke.particles.retain(|p| p.pattern < 5);
    });
}

pub fn get_smoke_particles() -> Vec<Particle> {
    SMOKE.with(|s| {
        let smoke = s.borrow();
        smoke.particles.iter().cloned().collect()
    })
}

pub fn clear_smoke() {
    SMOKE.with(|s| {
        let mut smoke = s.borrow_mut();
        smoke.particles.clear();
    });
}
