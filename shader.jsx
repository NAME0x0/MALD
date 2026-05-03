// WebGL shader hero — sage scanlines + grain + vignette + drifting glow band
// Renders at 30fps, pauses when tab hidden or prefers-reduced-motion.

const VERT = `
attribute vec2 aPos;
varying vec2 vUv;
void main() {
  vUv = (aPos + 1.0) * 0.5;
  gl_Position = vec4(aPos, 0.0, 1.0);
}
`;

const FRAG = `
precision highp float;
varying vec2 vUv;
uniform float uTime;
uniform vec2 uResolution;
uniform float uIntensity;

float hash(vec2 p) {
  return fract(sin(dot(p, vec2(12.9898, 78.233))) * 43758.5453);
}

void main() {
  vec3 base = vec3(0.043, 0.051, 0.055); // --bg-base
  vec3 sage = vec3(0.561, 0.686, 0.561);
  vec3 color = base;

  // Horizontal scanlines drifting downward
  float line = sin((vUv.y + uTime * 0.06) * 800.0);
  float scan = smoothstep(0.96, 1.0, line) * 0.05 * uIntensity;
  color += sage * scan;

  // Film grain
  float grain = (hash(vUv * uResolution + uTime) - 0.5) * 0.04 * uIntensity;
  color += vec3(grain);

  // Vignette
  float v = smoothstep(0.55, 1.3, distance(vUv, vec2(0.5)));
  color *= 1.0 - v * 0.55;

  // Slow vertical sage glow band traversing every 12s
  float band = exp(-pow((vUv.y - fract(uTime * 0.083)) * 6.0, 2.0));
  color += sage * band * 0.045 * uIntensity;

  // Subtle horizontal drift band (extra atmosphere)
  float h = exp(-pow((vUv.x - 0.3) * 2.5, 2.0)) * 0.02;
  color += sage * h;

  gl_FragColor = vec4(color, 1.0);
}
`;

function compileShader(gl, type, src) {
  const sh = gl.createShader(type);
  gl.shaderSource(sh, src);
  gl.compileShader(sh);
  if (!gl.getShaderParameter(sh, gl.COMPILE_STATUS)) {
    console.error('Shader error:', gl.getShaderInfoLog(sh));
    gl.deleteShader(sh);
    return null;
  }
  return sh;
}

function ShaderBackground({ intensity = 1 }) {
  const canvasRef = React.useRef(null);
  const stateRef = React.useRef({});
  const reduced = React.useRef(window.matchMedia('(prefers-reduced-motion: reduce)').matches);

  React.useEffect(() => {
    const canvas = canvasRef.current;
    const gl = canvas.getContext('webgl', { antialias: false, alpha: false });
    if (!gl) return;

    const vs = compileShader(gl, gl.VERTEX_SHADER, VERT);
    const fs = compileShader(gl, gl.FRAGMENT_SHADER, FRAG);
    const prog = gl.createProgram();
    gl.attachShader(prog, vs);
    gl.attachShader(prog, fs);
    gl.linkProgram(prog);
    gl.useProgram(prog);

    const buf = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buf);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([
      -1, -1,  1, -1,  -1, 1,
      -1,  1,  1, -1,   1, 1,
    ]), gl.STATIC_DRAW);

    const aPos = gl.getAttribLocation(prog, 'aPos');
    gl.enableVertexAttribArray(aPos);
    gl.vertexAttribPointer(aPos, 2, gl.FLOAT, false, 0, 0);

    const uTime = gl.getUniformLocation(prog, 'uTime');
    const uRes = gl.getUniformLocation(prog, 'uResolution');
    const uInt = gl.getUniformLocation(prog, 'uIntensity');

    const dpr = Math.min(window.devicePixelRatio || 1, 1.5);
    function resize() {
      const w = canvas.clientWidth;
      const h = canvas.clientHeight;
      canvas.width = Math.floor(w * dpr);
      canvas.height = Math.floor(h * dpr);
      gl.viewport(0, 0, canvas.width, canvas.height);
      gl.uniform2f(uRes, canvas.width, canvas.height);
    }
    resize();
    const ro = new ResizeObserver(resize);
    ro.observe(canvas);

    let raf;
    let last = 0;
    let start = performance.now();
    const minFrame = 1000 / 30;
    let running = true;

    function frame(now) {
      if (!running) return;
      if (now - last >= minFrame) {
        const t = (now - start) / 1000;
        gl.uniform1f(uTime, reduced.current ? 0 : t);
        gl.uniform1f(uInt, intensity);
        gl.drawArrays(gl.TRIANGLES, 0, 6);
        last = now;
      }
      raf = requestAnimationFrame(frame);
    }

    if (reduced.current) {
      // Static frame
      gl.uniform1f(uTime, 0);
      gl.uniform1f(uInt, intensity);
      gl.drawArrays(gl.TRIANGLES, 0, 6);
    } else {
      raf = requestAnimationFrame(frame);
    }

    function onVis() {
      if (document.visibilityState === 'hidden') {
        running = false;
        cancelAnimationFrame(raf);
      } else if (!reduced.current) {
        running = true;
        raf = requestAnimationFrame(frame);
      }
    }
    document.addEventListener('visibilitychange', onVis);

    stateRef.current = { gl, prog, uInt, redraw: () => {
      gl.uniform1f(uInt, stateRef.current.intensity ?? 1);
    }};

    return () => {
      running = false;
      cancelAnimationFrame(raf);
      ro.disconnect();
      document.removeEventListener('visibilitychange', onVis);
    };
  }, []);

  React.useEffect(() => {
    stateRef.current.intensity = intensity;
  }, [intensity]);

  return (
    <canvas
      ref={canvasRef}
      style={{
        position: 'absolute',
        inset: 0,
        width: '100%',
        height: '100%',
        display: 'block',
        zIndex: 0,
      }}
    />
  );
}

window.ShaderBackground = ShaderBackground;
