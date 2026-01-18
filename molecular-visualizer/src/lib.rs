// Copyright (c) 2026 Valery Vishnevskiy and Yury Vishnevskiy
// Licensed under the MIT License

#[allow(warnings)]
mod bindings {
    wit_bindgen::generate!({
        path: "wit",
        world: "plugin",
    });

    use super::ChemistryMoleculeVisualizer;

    export!(ChemistryMoleculeVisualizer);
}

use bindings::Guest;

struct ChemistryMoleculeVisualizer;

impl Guest for ChemistryMoleculeVisualizer {
    fn render() -> String {
        r##"
<div style="display:flex;flex-direction:column;align-items:center;padding:10px;font-family:system-ui,sans-serif;">
    <canvas id="moleculeCanvas" width="320" height="280"></canvas>
    <div style="margin-top:8px;font-size:14px;color:#333;font-weight:600;">3D Atom Model</div>
    <div style="font-size:12px;color:#666;">Electrons orbiting nucleus</div>
</div>
<script>
(function() {
    const canvas = document.getElementById('moleculeCanvas');
    const ctx = canvas.getContext('2d');
    const W = canvas.width, H = canvas.height;
    const cx = W / 2, cy = H / 2;
    
    const electrons = [
        { orbit: 50, speed: 0.03, angle: 0, tilt: 0.3, color: '#4fc3f7' },
        { orbit: 50, speed: 0.03, angle: Math.PI, tilt: 0.3, color: '#4fc3f7' },
        { orbit: 80, speed: 0.02, angle: 0, tilt: -0.5, color: '#81c784' },
        { orbit: 80, speed: 0.02, angle: 2.09, tilt: -0.5, color: '#81c784' },
        { orbit: 80, speed: 0.02, angle: 4.19, tilt: -0.5, color: '#81c784' },
        { orbit: 110, speed: 0.012, angle: 0, tilt: 0.7, color: '#ffb74d' },
        { orbit: 110, speed: 0.012, angle: 1.57, tilt: 0.7, color: '#ffb74d' },
        { orbit: 110, speed: 0.012, angle: 3.14, tilt: 0.7, color: '#ffb74d' },
        { orbit: 110, speed: 0.012, angle: 4.71, tilt: 0.7, color: '#ffb74d' },
    ];
    
    function drawNucleus() {
        const gradient = ctx.createRadialGradient(cx - 5, cy - 5, 0, cx, cy, 25);
        gradient.addColorStop(0, '#ff8a80');
        gradient.addColorStop(0.5, '#f44336');
        gradient.addColorStop(1, '#b71c1c');
        ctx.beginPath();
        ctx.arc(cx, cy, 22, 0, Math.PI * 2);
        ctx.fillStyle = gradient;
        ctx.fill();
        ctx.shadowColor = '#f44336';
        ctx.shadowBlur = 20;
        ctx.fill();
        ctx.shadowBlur = 0;
    }
    
    function drawOrbit(radius, tilt, alpha) {
        ctx.beginPath();
        ctx.ellipse(cx, cy, radius, radius * Math.abs(Math.cos(tilt * Math.PI)), 0, 0, Math.PI * 2);
        ctx.strokeStyle = 'rgba(255,255,255,' + alpha + ')';
        ctx.lineWidth = 1;
        ctx.stroke();
    }
    
    function drawElectron(e) {
        const x = cx + Math.cos(e.angle) * e.orbit;
        const yBase = Math.sin(e.angle) * e.orbit * Math.cos(e.tilt * Math.PI);
        const y = cy + yBase;
        const z = Math.sin(e.angle) * Math.sin(e.tilt * Math.PI);
        const size = 6 + z * 2;
        
        const gradient = ctx.createRadialGradient(x - 2, y - 2, 0, x, y, size);
        gradient.addColorStop(0, '#fff');
        gradient.addColorStop(0.3, e.color);
        gradient.addColorStop(1, e.color.replace(')', ', 0.3)').replace('rgb', 'rgba'));
        
        ctx.beginPath();
        ctx.arc(x, y, Math.max(3, size), 0, Math.PI * 2);
        ctx.fillStyle = gradient;
        ctx.fill();
        
        ctx.shadowColor = e.color;
        ctx.shadowBlur = 10;
        ctx.fill();
        ctx.shadowBlur = 0;
    }
    
    function drawTrail(e) {
        for (let i = 1; i <= 8; i++) {
            const trailAngle = e.angle - i * 0.08;
            const x = cx + Math.cos(trailAngle) * e.orbit;
            const y = cy + Math.sin(trailAngle) * e.orbit * Math.cos(e.tilt * Math.PI);
            const alpha = 0.3 - i * 0.035;
            ctx.beginPath();
            ctx.arc(x, y, 3 - i * 0.3, 0, Math.PI * 2);
            ctx.fillStyle = e.color.replace(')', ',' + alpha + ')').replace('rgb', 'rgba').replace('#', 'rgba(').replace(/([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]{2})/i, function(m, r, g, b) {
                return parseInt(r, 16) + ',' + parseInt(g, 16) + ',' + parseInt(b, 16);
            });
            ctx.fill();
        }
    }
    
    function animate() {
        ctx.fillStyle = 'rgba(30, 30, 40, 1)';
        ctx.fillRect(0, 0, W, H);
        
        drawOrbit(50, 0.3, 0.15);
        drawOrbit(80, -0.5, 0.12);
        drawOrbit(110, 0.7, 0.1);
        
        const sorted = [...electrons].sort((a, b) => {
            const za = Math.sin(a.angle) * Math.sin(a.tilt * Math.PI);
            const zb = Math.sin(b.angle) * Math.sin(b.tilt * Math.PI);
            return za - zb;
        });
        
        const backElectrons = sorted.filter(e => Math.sin(e.angle) * Math.sin(e.tilt * Math.PI) < 0);
        backElectrons.forEach(e => { drawTrail(e); drawElectron(e); });
        
        drawNucleus();
        
        const frontElectrons = sorted.filter(e => Math.sin(e.angle) * Math.sin(e.tilt * Math.PI) >= 0);
        frontElectrons.forEach(e => { drawTrail(e); drawElectron(e); });
        
        electrons.forEach(e => { e.angle += e.speed; });
        
        requestAnimationFrame(animate);
    }
    
    animate();
})();
</script>
        "##.to_string()
    }
}
