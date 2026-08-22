struct SimParams {
  dt: f32,
  count: u32,
}

struct Particle {
  pos: vec3<f32>,
  vel: vec3<f32>,
}

@group(0) @binding(0) var<uniform> params: SimParams;
@group(0) @binding(1) var<storage, read_write> particles: array<Particle>;

fn integrate(particle: Particle, dt: f32) -> Particle {
  let speed = length(particle.vel);
  if (speed > 0.0f) {
    var pos = particle.pos + particle.vel * dt;
    return Particle(pos, particle.vel);
  }
  return particle;
}

@compute @workgroup_size(64, 1, 1)
fn particleKernel(@builtin(global_invocation_id) globalId: vec3<u32>) {
  var settings = params;
  let i = globalId.x;
  if (i < settings.count) {
    particles[i] = integrate(particles[i], settings.dt);
  }
}
