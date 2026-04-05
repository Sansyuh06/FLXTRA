"use client"

import { useEffect, useRef } from "react"

// ─── Skeletal anatomy (total height ≈ 62px standing) ─────────────────────────
const HR = 10    // head radius
const NL = 7    // neck length
const SP = 22   // spine: hip → neck
const UA = 13   // upper arm: shoulder → elbow
const LA = 11   // lower arm: elbow → hand
const UL = 15   // upper leg: hip → knee
const LL = 15   // lower leg: knee → foot

const RED  = "#ef4444"
const GLOW_COL = "#ef4444"

interface V2 { x: number; y: number }

interface Pose {
  head: V2; neck: V2
  sL: V2; sR: V2
  eL: V2; eR: V2
  hL: V2; hR: V2
  hip: V2
  kL: V2; kR: V2
  fL: V2; fR: V2
}

interface Platform {
  top: number; bottom: number
  left: number; right: number
  width: number; height: number
  kind: "surface" | "wall" | "ledge"
  label: string
  lastVisited: number
}

interface Particle {
  x: number; y: number; vx: number; vy: number
  life: number; size: number; type: "dust" | "star"
}

type State =
  | "RUN" | "WALK" | "IDLE" | "JUMP" | "FALL" | "LAND"
  | "HANG" | "WALLRUN" | "BACKFLIP" | "VAULT" | "SLIDE"
  | "DIVEROLL" | "CATCHBREATH" | "TIC_TAC" | "PRECISION_JUMP"
  | "CURSOR_CHASE" | "CURSOR_HELD" | "CURSOR_FLUNG" | "WAVE" | "HIDDEN" | "SIT"

export function ParkourStickman() {
  const cvs = useRef<HTMLCanvasElement>(null)

  useEffect(() => {
    const canvas = cvs.current
    if (!canvas) return
    const ctx = canvas.getContext("2d")!

    let W = window.innerWidth
    let H = window.innerHeight
    canvas.width = W; canvas.height = H

    // ── Math helpers ──────────────────────────────────────────────────────────
    const lerp  = (a: number, b: number, t: number) => a + (b - a) * t
    const clamp = (v: number, lo: number, hi: number) => Math.max(lo, Math.min(hi, v))
    const lerpV = (a: V2, b: V2, t: number): V2 => ({
      x: lerp(a.x, b.x, t), y: lerp(a.y, b.y, t)
    })

    const blendPose = (cur: Pose, tar: Pose, t: number): Pose => ({
      head: lerpV(cur.head, tar.head, t), neck: lerpV(cur.neck, tar.neck, t),
      sL: lerpV(cur.sL, tar.sL, t),      sR: lerpV(cur.sR, tar.sR, t),
      eL: lerpV(cur.eL, tar.eL, t),      eR: lerpV(cur.eR, tar.eR, t),
      hL: lerpV(cur.hL, tar.hL, t),      hR: lerpV(cur.hR, tar.hR, t),
      hip: lerpV(cur.hip, tar.hip, t),
      kL: lerpV(cur.kL, tar.kL, t),      kR: lerpV(cur.kR, tar.kR, t),
      fL: lerpV(cur.fL, tar.fL, t),      fR: lerpV(cur.fR, tar.fR, t),
    })

    const clonePose = (p: Pose): Pose => JSON.parse(JSON.stringify(p))

    const limitReach = (rx: number, ry: number, tx: number, ty: number, maxD: number): V2 => {
      const d = Math.max(0.001, Math.hypot(tx-rx, ty-ry))
      if (d > maxD) return { x: rx + ((tx-rx)/d)*maxD, y: ry + ((ty-ry)/d)*maxD }
      return { x: tx, y: ty }
    }

    // ── IK solver ─────────────────────────────────────────────────────────────
    const ik = (rx:number,ry:number, tx:number,ty:number, b1:number,b2:number, dir:number): V2 => {
      const dx = tx - rx, dy = ty - ry
      let d  = Math.max(0.001, Math.hypot(dx, dy))
      d = clamp(d, Math.abs(b1-b2)+0.5, b1+b2-0.5)
      const cosA = clamp((b1*b1 + d*d - b2*b2) / (2*b1*d), -1, 1)
      const alpha = Math.acos(cosA) || 0
      const ang   = Math.atan2(dy, dx) + dir * alpha
      return { x: rx + Math.cos(ang)*b1, y: ry + Math.sin(ang)*b1 }
    }

    const neutral = (hx: number, hy: number): Pose => {
      const neck = { x: hx, y: hy - SP }
      return {
        head: { x: hx, y: neck.y - NL - HR },
        neck,
        sL:{ x:hx-6, y:neck.y }, sR:{ x:hx+6, y:neck.y },
        eL:{ x:hx-9, y:neck.y+UA*0.6 }, eR:{ x:hx+9, y:neck.y+UA*0.6 },
        hL:{ x:hx-7, y:neck.y+UA+LA*0.5 }, hR:{ x:hx+7, y:neck.y+UA+LA*0.5 },
        hip:{ x:hx, y:hy },
        kL:{ x:hx-5, y:hy+UL }, kR:{ x:hx+5, y:hy+UL },
        fL:{ x:hx-6, y:hy+UL+LL }, fR:{ x:hx+6, y:hy+UL+LL },
      }
    }

    let platforms: Platform[]  = []
    let textPlatforms: Platform[] = []
    let initialized = false

    const extractGeometry = () => {
      platforms = []
      document.querySelectorAll("h1, h2, button, span, a").forEach(el => {
        const elRect = el.getBoundingClientRect()
        if (elRect.width < 10 || elRect.height < 3) return
        let gotWords = false
        const walker = document.createTreeWalker(el, NodeFilter.SHOW_TEXT)
        let tn: Text | null
        while ((tn = walker.nextNode() as Text)) {
          const raw = tn.textContent ?? ""
          if (!raw.trim()) continue
          let at = 0
          raw.split(/\s+/).filter(w => w.length > 0).forEach(word => {
            const idx = raw.indexOf(word, at)
            if (idx < 0) return
            at = idx + word.length
            try {
              const range = document.createRange()
              range.setStart(tn!, idx)
              range.setEnd(tn!, idx + word.length)
              Array.from(range.getClientRects()).forEach(rect => {
                if (rect.width < 8 || rect.height < 6) return
                gotWords = true
                let kind: Platform["kind"] = "ledge"
                if (rect.width > rect.height * 1.8)  kind = "surface"
                else if (rect.height > rect.width * 1.2) kind = "wall"
                platforms.push({
                  top: rect.top, bottom: rect.bottom,
                  left: rect.left, right: rect.right,
                  width: rect.width, height: rect.height,
                  kind, label: word, lastVisited: 0
                })
              })
            } catch { /* ignore */ }
          })
        }
        if (!gotWords && elRect.width > 10 && !["MAIN", "BODY", "HTML", "DIV", "SECTION", "NAV", "FORM", "ARTICLE", "ASIDE", "HEADER", "FOOTER"].includes(el.tagName.toUpperCase())) {
          platforms.push({
            top: elRect.top, bottom: elRect.bottom,
            left: elRect.left, right: elRect.right,
            width: elRect.width, height: elRect.height,
            kind: "surface", label: el.tagName, lastVisited: 0
          })
        }
      })
      textPlatforms = [...platforms]
      platforms.push({
        top: H - 2, bottom: H, left: -500, right: W + 500,
        width: W + 1000, height: 2, kind: "surface", label: "ground", lastVisited: 0
      })
      if (textPlatforms.length > 0 && !initialized) {
        initPosition()
        initialized = true
      }
    }

    const initPosition = () => {
      hipX = Math.random() < 0.5 ? -30 : W + 30
      hipY = -100
      groundY = H
      grounded = false
      vy = 0; vx = hipX < W/2 ? 3 : -3
      _curPose = neutral(hipX, hipY)
      go("FALL")
    }

    let state: State = "FALL"
    let timer = 0
    let actionDuration = 800

    let hipX = W / 2, hipY = H / 2
    let vx = 0, vy = 0
    let facing = 1
    let grounded = false
    let groundY  = H - 2

    let mouseX = W/2, mouseY = H/2
    let prevMX = W/2, prevMY = H/2
    let mouseVX = 0, mouseVY = 0
    let shakeScore = 0
    let cursorCooldown = 0
    let cursorNear = false
    let activeTime = 0

    let hangPlat: Platform | null = null
    let hangBarX = 0, hangBarY = 0
    let interactPlat: Platform | null = null
    let targetDestX = 0, targetDestY = 0
    let flipAngle = 0

    let history: State[] = []
    const recentDid = (s: State, n = 3) => history.slice(-n).includes(s)
    let lookWaitTime = 0

    const particles: Particle[] = []
    const trail: Pose[] = []
    let _curPose = neutral(hipX, hipY)

    // Hand-Drawn Soft Body Physics Variables
    let softSquashY = 1.0
    let softSquashX = 1.0
    let hairSwayX = 0
    let hairSwayV = 0

    const spawnDust = (x: number, y: number, n = 4) => {
      for (let i = 0; i < n; i++) {
        const a = -Math.PI + (Math.random()-0.5)*1.4
        const s = 1 + Math.random()*2.5
        particles.push({ x, y, vx:Math.cos(a)*s, vy:Math.sin(a)*s-0.5, life:1, size:0, type:"dust" })
      }
    }

    const spawnStars = (x: number, y: number) => {
      for (let i = 0; i < 7; i++) {
        const a = (i/7)*Math.PI*2
        const s = 2.5 + Math.random()*3
        particles.push({ x, y, vx:Math.cos(a)*s, vy:Math.sin(a)*s, life:1, size:2+Math.random()*2, type:"star" })
      }
    }

    const go = (s: State) => {
      if (s === state) return
      history.push(state)
      if (history.length > 8) history.shift()
      state = s
      timer = 0
      switch (s) {
        case "RUN":         actionDuration = 500 + Math.random() * 800; break // Short repositioning sprints
        case "WALK":        actionDuration = 1500 + Math.random() * 1500; break
        case "IDLE":        actionDuration = 4000 + Math.random() * 6000; break // Stand still longer
        case "CATCHBREATH": actionDuration = 3000 + Math.random() * 3000; break
        case "LAND":        actionDuration = 200; break
        case "SLIDE":       actionDuration = 700 + Math.random() * 300; break
        case "VAULT":       actionDuration = 600; break
        case "DIVEROLL":    actionDuration = 700; break
        case "BACKFLIP":    actionDuration = 600; break
        case "WALLRUN":     actionDuration = 500; break
        case "TIC_TAC":     actionDuration = 400; break
        case "HANG":        actionDuration = 3000 + Math.random() * 4000; break // Hang longer
        case "PRECISION_JUMP": actionDuration = 800; break
        case "WAVE":        actionDuration = 3000 + Math.random() * 1500; break // Longer, friendlier wave
        case "HIDDEN":      actionDuration = 10000; break // 10 seconds hiding!
        case "SIT":         actionDuration = 5000 + Math.random() * 5000; break // Laptop session
        default:            actionDuration = 2000; break
      }
    }

    const GAIT = [
      { lL:-50, rL: 35, lA: 55, rA:-45, lean:0.4, bob:-2 },
      { lL:-30, rL: 55, lA: 35, rA:-55, lean:0.45, bob: 4 },
      { lL: 15, rL: 60, lA:-15, rA:-60, lean:0.5, bob:-2 },
      { lL: 45, rL: 30, lA:-45, rA:-30, lean:0.55, bob:-4 },
      { lL: 60, rL:-15, lA:-60, rA: 15, lean:0.5, bob:-2 },
      { lL: 55, rL:-30, lA:-55, rA: 30, lean:0.45, bob: 4 },
      { lL: 35, rL:-50, lA:-35, rA: 50, lean:0.4, bob:-2 },
    ]

    const WALK_GAIT = [
      { lL:-35, rL: 25, lA: 30, rA:-30 },
      { lL:-15, rL: 35, lA: 10, rA:-35 },
      { lL: 15, rL: 25, lA:-15, rA:-25 },
      { lL: 35, rL: 10, lA:-35, rA:-10 },
      { lL: 25, rL:-15, lA:-25, rA: 15 },
      { lL: 15, rL:-35, lA:-10, rA: 35 },
    ]

    const GRAVITY = 0.5

    const jumpFor = (targetY: number): number => {
      const rise = (hipY + UL + LL) - targetY + 25
      if (rise <= 0) return -3.5
      return -Math.sqrt(2 * GRAVITY * rise) - 1.5
    }

    const plan = () => {
      // Allow fluid transition across states. Decide randomly if we should traverse or seek text.
      const runMode = Math.random() < 0.6 ? "FREE_ROAM" : "SEEK_TEXT"
      const now = Date.now()

      if (cursorCooldown <= 0 && cursorNear) {
         go("CURSOR_CHASE"); return
      }

      // Tighter random generation restricted around text boxes
      let minX = Math.max(100, W/3)
      let maxX = Math.min(W - 100, W * 2/3)
      textPlatforms.forEach(p => { 
        minX = Math.min(minX, Math.max(100, p.left - 100))
        maxX = Math.max(maxX, Math.min(W - 100, p.right + 100)) 
      })
      if (minX > maxX) { minX = W/2 - 50; maxX = W/2 + 50 }
      const cx = (minX + maxX) / 2
      const spread = (maxX - minX) / 2

      if (Math.random() < 0.05 && !recentDid("FALL")) {
        // Jump from sky directly above the text
        hipX = cx + (Math.random() > 0.5 ? 1 : -1) * (Math.random() * spread)
        hipY = -100
        groundY = H
        grounded = false
        vy = Math.random() * 5
        vx = (cx - hipX > 0 ? 1 : -1) * (1 + Math.random()*3)
        facing = vx >= 0 ? 1 : -1
        go("FALL")
        return
      }

      if (runMode === "FREE_ROAM") {
        // Patrol tightly around the central text coordinates, don't run to absolute edges
        targetDestX = Math.max(minX, Math.min(maxX, cx + (Math.random() > 0.5 ? 1 : -1) * (spread*0.8 + Math.random()*150)))
        facing = targetDestX > hipX ? 1 : -1
        vx = facing * (3 + Math.random()*3)
        const moves = ["RUN", "WALK", "SLIDE", "DIVEROLL", "JUMP"]
        const nxt = moves[Math.floor(Math.random()*moves.length)] as State
        if (nxt === "SLIDE" || nxt === "DIVEROLL") {
          vx = facing * 6
          if (!recentDid(nxt)) { go(nxt); return }
        }
        if (nxt === "JUMP") {
           vy = -4; go("JUMP"); return
        }
        go(Math.random() > 0.2 ? "RUN" : "WALK")
        return
      }

      // Seek text platform
      const scored = textPlatforms.map(p => {
        const cx = (p.left + p.right) / 2
        const hdist = Math.abs(cx - hipX)
        const vdist = Math.abs(p.top - hipY)
        let sc = 3000 / (hdist + vdist + 1)
        if (p.top < hipY - 10) sc *= 5.0 // Prefer going UP
        const ago = now - p.lastVisited
        if (ago < 4000) sc *= 0.01
        return { p, sc, cx }
      }).sort((a,b) => b.sc - a.sc)

      const best = scored[0]
      if (best && best.sc > 0.1) {
        const { p, cx } = best
        p.lastVisited = now
        facing = cx >= hipX ? 1 : -1

        if (p.top < hipY - 5 && Math.abs(cx - hipX) < 350) {
          vy = jumpFor(p.top)
          vx = facing * clamp(Math.abs(cx - hipX) / 30, 2, 6.5)
          grounded = false
          go(Math.random() < 0.2 ? "PRECISION_JUMP" : "JUMP")
          return
        }

        if (Math.abs(p.top - (hipY + UL + LL)) < 50) {
          if (!recentDid("VAULT", 2) && p.width < 120 && Math.random() < 0.5) { interactPlat = p; go("VAULT"); return }
          if (!recentDid("SLIDE", 3) && Math.random() < 0.2) { go("SLIDE"); return }
          targetDestX = cx
          go(Math.random() < 0.6 ? "RUN" : "WALK")
          return
        }

        if (p.top > hipY + UL + LL + 10) {
          vy = -3.5; vx = facing * 3; grounded = false
          go(Math.random() < 0.3 ? "DIVEROLL" : "JUMP")
          return
        }
      }

      // Fallback - strictly limit hyperactivity by occasionally doing laptop work
      if (!recentDid("IDLE") && Math.random() < 0.6) { go("IDLE"); return }
      if (!recentDid("SIT") && Math.random() < 0.4) { go("SIT"); return }
      if (!recentDid("WALK") && Math.random() < 0.6) {
        facing = Math.random() > 0.5 ? 1 : -1
        vx = facing * 2.5
        go("WALK")
        return
      }
      facing = Math.random() > 0.5 ? 1 : -1
      vx = facing * 4
      go("RUN")
    }

    const checkLand = () => {
      if (["HANG","WALLRUN","TIC_TAC","CURSOR_HELD"].includes(state)) return
      const fY = hipY + UL + LL
      const prevFY = fY - vy

      for (const p of platforms) {
        if (p.kind !== "surface" && p.kind !== "ledge") continue
        if (hipX < p.left - 18 || hipX > p.right + 18) continue
        if (prevFY <= p.top + 6 && fY >= p.top && vy >= 0) {
          hipY = p.top - UL - LL
          groundY = p.top
          vy = 0
          if (!grounded) {
            spawnDust(hipX, p.top, 6)
            
            // Extreme hand-drawn impact squash!
            if (prevFY - fY > 10) { softSquashY = 0.5; softSquashX = 1.4 }
            else if (prevFY - fY > 4) { softSquashY = 0.75; softSquashX = 1.2 }

            p.lastVisited = Date.now()
            flipAngle = 0
            if (["JUMP","FALL","PRECISION_JUMP","DIVEROLL"].includes(state)) {
              if (state === "DIVEROLL") go("RUN")
              else if (state === "PRECISION_JUMP") { vx=0; go("CATCHBREATH") } // lands with emphasis
              else go("LAND")
            }
          }
          grounded = true
          return
        }
      }

      const footY2 = hipY + UL + LL
      const stillOn = platforms.some(p =>
        (p.kind==="surface"||p.kind==="ledge") &&
        hipX >= p.left-18 && hipX <= p.right+18 &&
        Math.abs(footY2 - p.top) < 12
      )

      if (!stillOn && grounded) {
        grounded = false
        if (["RUN","WALK","IDLE","SLIDE","VAULT"].includes(state)) go("FALL")
      }
    }

    let hangCooldown = 0
    const checkHang = () => {
      if (hangCooldown > 0 || !["JUMP","FALL"].includes(state) || vy > 5) return
      const handY = hipY - SP - UA * 0.2
      for (const p of textPlatforms) {
        if (p.width < 15 || p === hangPlat) continue
        if (Math.abs(handY - p.top) < 22 && hipX >= p.left-15 && hipX <= p.right+15 && Math.random() < 0.6) {
          hangPlat = p
          hangBarX = clamp(hipX, p.left+12, p.right-12)
          hangBarY = p.top
          hipX = hangBarX
          hipY = hangBarY + UA + SP + 10 // dangling lower
          vx = 0; vy = 0; grounded = false
          p.lastVisited = Date.now()
          spawnStars(hangBarX, hangBarY)
          go("HANG")
          return
        }
      }
    }

    // Dynamic obstacle detection during locomotion
    const checkObstacles = () => {
      if (!grounded || !["RUN","WALK"].includes(state)) return
      const fY = hipY + UL + LL
      const headY = hipY - SP - HR - 10
      for (const p of textPlatforms) {
        const dist = facing === 1 ? p.left - hipX : hipX - p.right
        if (dist > 5 && dist < 65) { // approaching horizontally
          // Check if object is physically blocking the body vertically
          const isBlockingBody = p.bottom > headY && p.top < fY + 5
          if (!isBlockingBody) continue

          const obstacleHeightFromFloor = fY - p.top // positive means object top is above feet
          
          if (obstacleHeightFromFloor > 10 && obstacleHeightFromFloor < 65 && !recentDid("VAULT", 4)) {
            interactPlat = p; go("VAULT"); return
          } else if (obstacleHeightFromFloor >= 65 || p.kind === "wall") {
            interactPlat = p
            if (Math.random() < 0.5) go("WALLRUN")
            else go("TIC_TAC")
            return
          }
        }
      }
    }

    const drawCharBody = (p: Pose, ctx: CanvasRenderingContext2D) => {
      const currentFacing = facing 
      
      const drawShoe = (foot: V2) => {
        ctx.fillStyle = "#000"; ctx.beginPath()
        ctx.ellipse(foot.x + currentFacing*4, foot.y+2, 9, 5, 0, 0, Math.PI*2)
        ctx.fill()
      }

      const drawLeg = (hip: V2, knee: V2, foot: V2, color: string) => {
        const dx1 = knee.x - hip.x, dy1 = knee.y - hip.y
        const nx1 = -dy1 / (Math.hypot(dx1, dy1) || 1)
        const ny1 = dx1 / (Math.hypot(dx1, dy1) || 1)

        const dx2 = foot.x - knee.x, dy2 = foot.y - knee.y
        const nx2 = -dy2 / (Math.hypot(dx2, dy2) || 1)
        const ny2 = dx2 / (Math.hypot(dx2, dy2) || 1)

        // Ultra Loose Straight-Leg Cargo Pants (Very Baggy overall)
        const hw = 4.5, kw = 6, fw = 6.5
        
        const tX = (hip.x + knee.x)/2, tY = (hip.y + knee.y)/2
        const cX = (knee.x + foot.x)/2, cY = (knee.y + foot.y)/2

        ctx.fillStyle = color
        ctx.strokeStyle = "#000"
        ctx.lineWidth = 2
        ctx.lineJoin = "round"

        // Organic baggy cloth boundaries
        ctx.beginPath()
        ctx.moveTo(hip.x + nx1*hw, hip.y + ny1*hw)
        ctx.quadraticCurveTo(tX + nx1*(kw+1), tY + ny1*(kw+1), knee.x + nx1*kw, knee.y + ny1*kw)
        ctx.quadraticCurveTo(cX + nx2*(kw+1), cY + ny2*(kw+1), foot.x + nx2*fw, foot.y + ny2*fw)
        ctx.lineTo(foot.x - nx2*(fw+1), foot.y - ny2*(fw+1)) // slightly wider cuff at the shoe
        ctx.quadraticCurveTo(cX - nx2*(kw-2), cY - ny2*(kw-2), knee.x - nx1*kw, knee.y - ny1*kw)
        ctx.quadraticCurveTo(tX - nx1*(kw-2), tY - ny1*(kw-2), hip.x - nx1*hw, hip.y - ny1*hw)
        ctx.closePath()
        ctx.fill(); ctx.stroke()

        // Pockets & User's Textures Layer
        ctx.save()
        ctx.clip()
        
        // 1. User's Hand-Drawn Swirl Textures (Maintained as requested)
        ctx.beginPath()
        ctx.strokeStyle = "rgba(0,0,0,0.18)"
        ctx.lineWidth = 2
        ctx.moveTo(hip.x, hip.y); ctx.quadraticCurveTo(knee.x+9, (hip.y+knee.y)/2, knee.x, knee.y)
        ctx.moveTo(knee.x, knee.y); ctx.quadraticCurveTo(foot.x-10, (knee.y+foot.y)/2, foot.x, foot.y)
        ctx.moveTo(knee.x, knee.y); ctx.quadraticCurveTo(foot.x+10, (knee.y+foot.y)/2, foot.x, foot.y)
        ctx.stroke()
        
        // 2. Cargo Side-Pocket Mapped to Thigh Action Angle
        const thighAng = Math.atan2(knee.y - hip.y, knee.x - hip.x)
        ctx.translate(tX, tY)
        ctx.rotate(thighAng)
        
        ctx.fillStyle = "rgba(0,0,0,0.08)"
        ctx.strokeStyle = "rgba(0,0,0,0.35)"
        ctx.lineWidth = 1.5
        // Box pocket offset slightly forward to match side seam
        ctx.fillRect(-2, -7, 13, 11)
        ctx.strokeRect(-2, -7, 13, 11)
        // Cargo Flap
        ctx.fillStyle = "rgba(0,0,0,0.15)"
        ctx.fillRect(-2, -7, 13, 4)
        ctx.strokeRect(-2, -7, 13, 4)

        ctx.restore()

        drawShoe(foot)
      }

      const drawLaptop = (hand: V2) => {
        ctx.save()
        ctx.translate(hand.x, hand.y)
        ctx.scale(currentFacing, 1)
        
        ctx.fillStyle = "#777"; ctx.strokeStyle = "#000"; ctx.lineWidth = 1.5
        ctx.beginPath(); ctx.moveTo(-6, 2); ctx.lineTo(14, 2); ctx.lineTo(11, 6); ctx.lineTo(-9, 6); ctx.closePath()
        ctx.fill(); ctx.stroke()
        
        ctx.fillStyle = "#fff"; ctx.strokeStyle = "#000"; ctx.lineWidth = 1.5
        ctx.beginPath(); ctx.arc(2, 4, 3.5, 0, Math.PI*2); ctx.fill(); ctx.stroke()
        
        ctx.fillStyle = "#a3a3a3"
        ctx.beginPath(); ctx.moveTo(-6, 2); ctx.lineTo(14, 2); ctx.lineTo(10, -14); ctx.lineTo(-10, -14); ctx.closePath()
        ctx.fill(); ctx.stroke()
        
        ctx.fillStyle = "#111"; ctx.beginPath(); ctx.arc(-1, -6, 2.5, 0, Math.PI*2); ctx.fill()
        
        ctx.fillStyle = "#4285F4"; ctx.fillRect(-7, -11, 3.5, 3.5)
        ctx.fillStyle = "#fbbc05"; ctx.beginPath(); ctx.arc(5, -3, 1.5, 0, Math.PI*2); ctx.fill()
        ctx.fillStyle = "#ea4335"; ctx.beginPath(); ctx.arc(4, -10, 1.2, 0, Math.PI*2); ctx.fill()
        ctx.restore()
      }

      const drawArm = (shoulder: V2, elbow: V2, hand: V2, isRight: boolean) => {
        const isHoldingLaptop = isRight && ["IDLE", "SIT"].includes(state)

        // Arms are thin stick-like sleeves
        ctx.lineWidth = 6.5; ctx.strokeStyle = "#000"; ctx.lineCap = "round"; ctx.lineJoin = "round"
        ctx.beginPath(); ctx.moveTo(shoulder.x, shoulder.y); ctx.lineTo(elbow.x, elbow.y); ctx.lineTo(hand.x, hand.y); ctx.stroke()
        ctx.lineWidth = 3.5; ctx.strokeStyle = "#222"; ctx.stroke()

        if (isHoldingLaptop) {
          drawLaptop(hand)
        } else {
          ctx.fillStyle = "#fff"; ctx.strokeStyle = "#000"; ctx.lineWidth = 1.2
          ctx.beginPath(); ctx.arc(hand.x, hand.y, 4, 0, Math.PI*2); ctx.fill(); ctx.stroke()
          ctx.beginPath(); ctx.moveTo(hand.x, hand.y-1); ctx.lineTo(hand.x + currentFacing*5, hand.y-2); ctx.stroke()
          ctx.beginPath(); ctx.moveTo(hand.x, hand.y+1); ctx.lineTo(hand.x + currentFacing*4, hand.y+1); ctx.stroke()
        }
      }

      const drawTorso = () => {
        const neck = p.neck, hip = p.hip
        const dx = hip.x - neck.x, dy = hip.y - neck.y
        const len = Math.hypot(dx, dy) || 1
        const nx = -dy / len, ny = dx / len
        
        // Very slim stick-body for the sweater, only slightly widening at waist
        const nw = 3.5, hw = 5 
        
        ctx.fillStyle = "#222"
        ctx.strokeStyle = "#000"
        ctx.lineWidth = 2
        ctx.lineJoin = "round"

        ctx.beginPath()
        ctx.moveTo(neck.x + nx*nw, neck.y + ny*nw - 3)
        ctx.lineTo(hip.x + nx*hw, hip.y + ny*hw + 3)
        ctx.lineTo(hip.x - nx*hw, hip.y - ny*hw + 3)
        ctx.lineTo(neck.x - nx*nw, neck.y - ny*nw - 3)
        ctx.closePath()
        ctx.fill(); ctx.stroke()
      }

      const drawHead = () => {
        const hx = p.head.x
        const hy = p.head.y - 1
        const f = currentFacing
        
        ctx.fillStyle = "#fff"; ctx.strokeStyle = "#000"; ctx.lineWidth = 1.5
        ctx.beginPath(); ctx.arc(hx, hy, 11, 0, Math.PI*2); ctx.fill(); ctx.stroke()

        // Restored Original Hand-Drawn 5-Tuft Hair
        ctx.save()
        ctx.translate(hx, hy)
        ctx.rotate(hairSwayX * 0.05 * f)
        
        ctx.fillStyle = "#2c0e0e" 
        ctx.lineWidth = 2
        ctx.strokeStyle = "#000"
        ctx.lineJoin = "round"
        ctx.lineCap = "round"
        ctx.beginPath()
        
        ctx.moveTo(-f*10, 2) 
        // Spike 1: Backwards tuft
        ctx.lineTo(-f*13, -2) 
        ctx.lineTo(-f*9,  -6)  
        // Spike 2: High back spike
        ctx.lineTo(-f*10, -14)
        ctx.lineTo(-f*4,  -12) 
        // Spike 3: Main high crown spike
        ctx.lineTo(f*1,  -17)
        ctx.lineTo(f*5,  -13) 
        // Spike 4: Forward top spike
        ctx.lineTo(f*10, -15) 
        ctx.lineTo(f*9,  -9) 
        // Spike 5: Forehead bang
        ctx.lineTo(f*13, -3) 
        ctx.lineTo(f*8,   2) 
        
        ctx.stroke() 
        ctx.lineTo(0, 0)
        ctx.closePath()
        ctx.fill()
        
        ctx.restore()

        // Glasses with distinct bridge
        const r = 3.8
        const cL = hx + f * 1
        const cR = hx + f * 7.5 
        ctx.strokeStyle = "#000"; ctx.fillStyle = "#fff"; ctx.lineWidth = 1.4
        // Bridge line
        ctx.beginPath(); ctx.moveTo(cL, hy-1.5); ctx.lineTo(cR, hy-1.5); ctx.stroke()
        // White circles
        ctx.beginPath(); ctx.arc(cL, hy - 1.5, r, 0, Math.PI*2); ctx.fill(); ctx.stroke()
        ctx.beginPath(); ctx.arc(cR, hy - 1.5, r, 0, Math.PI*2); ctx.fill(); ctx.stroke()

        // Fixed Moustache & Goatee: Thin faint grey scratches under the glasses
        ctx.strokeStyle = "rgba(0,0,0,0.5)"; ctx.lineWidth = 1.2
        ctx.beginPath()
        ctx.moveTo(hx + f*1.5, hy+5); ctx.lineTo(hx + f*5, hy+5) 
        ctx.moveTo(hx + f*2.5, hy+9); ctx.lineTo(hx + f*4, hy+10) 
        ctx.stroke()
      }

      // Original light-gray canvas colors for pants
      const backLeg  = currentFacing === 1 ? { k: p.kL, f: p.fL, c: '#9a9a9a' } : { k: p.kR, f: p.fR, c: '#9a9a9a' }
      const frontLeg = currentFacing === 1 ? { k: p.kR, f: p.fR, c: '#b5b5b5' } : { k: p.kL, f: p.fL, c: '#b5b5b5' }
      
      const backArm  = currentFacing === 1 ? { s: p.sL, e: p.eL, h: p.hL, isRight: false } : { s: p.sR, e: p.eR, h: p.hR, isRight: true }
      const frontArm = currentFacing === 1 ? { s: p.sR, e: p.eR, h: p.hR, isRight: true } : { s: p.sL, e: p.eL, h: p.hL, isRight: false }

      // ── Soft-Body Global Transformation ───────────────────────────────────────
      ctx.save()
      
      // We translate exactly to the hip, apply dynamic scaling for squash-and-stretch, then pull back!
      ctx.translate(p.hip.x, p.hip.y)
      ctx.scale(softSquashX, softSquashY)
      ctx.translate(-p.hip.x, -p.hip.y)

      drawLeg(p.hip, backLeg.k, backLeg.f, backLeg.c)
      drawArm(backArm.s, backArm.e, backArm.h, backArm.isRight)
      drawTorso()
      drawLeg(p.hip, frontLeg.k, frontLeg.f, frontLeg.c)
      drawHead()
      drawArm(frontArm.s, frontArm.e, frontArm.h, frontArm.isRight)
      
      ctx.restore() // Clear transformation
    }

    const drawChar = (p: Pose, alpha = 1, glow = false, spin = 0) => {
      ctx.save()
      ctx.globalAlpha = alpha
      ctx.lineCap = "round"; ctx.lineJoin = "round"

      if (spin !== 0) {
        ctx.translate(p.hip.x, p.hip.y)
        ctx.rotate(spin)
        ctx.translate(-p.hip.x, -p.hip.y)
      }

      drawCharBody(p, ctx)
      ctx.restore()
    }

    let lastT = performance.now()
    let rafId = 0
    let groundDwell = 0
    let legPhase = 0

    const tick = (now: number) => {
      const dt = Math.min(now - lastT, 40)
      lastT = now
      const n = dt / 16

      ctx.clearRect(0, 0, W, H)
      timer += dt
      legPhase += Math.max(0.5, Math.abs(vx)) * dt * 0.015

      if (state === "HIDDEN") {
         if (timer > actionDuration) {
           activeTime = 0
           const cx = W/2, spread = W/3
           if (Math.random() < 0.5) {
             hipX = cx + (Math.random() > 0.5 ? 1 : -1) * (Math.random() * spread)
             hipY = -100; vx = (cx - hipX > 0 ? 1 : -1) * (3 + Math.random()*3)
             vy = 2; grounded = false; go("FALL")
           } else {
             hipY = groundY; hipX = Math.random() < 0.5 ? -30 : W+30
             vx = (hipX < W/2 ? 1 : -1) * (3 + Math.random()*4)
             facing = vx > 0 ? 1 : -1; go("RUN")
           }
         }
         rafId = requestAnimationFrame(tick)
         return
      }

      activeTime += dt
      hangCooldown = Math.max(0, hangCooldown - dt)
      lookWaitTime = Math.max(0, lookWaitTime - dt)

      mouseVX = mouseX - prevMX; mouseVY = mouseY - prevMY
      prevMX = mouseX; prevMY = mouseY
      const cspeed = Math.hypot(mouseVX, mouseVY)
      shakeScore = clamp(shakeScore + Math.max(0, cspeed-12)*0.4 - 1.8*n, 0, 120)
      cursorCooldown = Math.max(0, cursorCooldown - dt)
      
      const distToCursor = Math.hypot(mouseX-hipX, mouseY-hipY)
      cursorNear = distToCursor < 120

      // Instant interact if directly hovered
      if (cursorCooldown <= 0 && distToCursor < 50 && !["CURSOR_HELD", "CURSOR_FLUNG", "HIDDEN"].includes(state)) {
         spawnStars(mouseX, mouseY)
         go("CURSOR_HELD")
      }

      const frozen = ["HANG","WALLRUN","TIC_TAC","CURSOR_HELD"].includes(state)
      if (!frozen) {
        if (!grounded) { vy += GRAVITY * n; vy = Math.min(vy, 22) }
        hipX += vx * n
        hipY += vy * n
      }

      if (grounded && !["RUN","WALK","SLIDE","VAULT","DIVEROLL"].includes(state)) {
        vx *= Math.pow(0.75, n)
      }

      checkLand()
      checkHang()
      checkObstacles()

      // Organic Soft-Body Momentum Processing
      // Hair Spring (drags backward based on horizontal velocity)
      const hairTargetX = -vx * 1.5 
      hairSwayV += (hairTargetX - hairSwayX) * 0.15
      hairSwayV *= 0.8  // dampening to prevent endless wobble
      hairSwayX += hairSwayV

      // Core Squash and Stretch depending on Air-Time
      let targetSquashY = 1.0 + Math.abs(vy) * 0.015 // stretch downwards when falling
      let targetSquashX = 1.0 - Math.abs(vy) * 0.005 // get thinner
      if (["SLIDE", "DIVEROLL", "SIT"].includes(state)) {
         targetSquashY = 0.85
         targetSquashX = 1.15
      }

      // Smooth restoration to targets
      softSquashY += (targetSquashY - softSquashY) * 0.25
      softSquashX += (targetSquashX - softSquashX) * 0.25

      if (grounded && groundY > H - 100) {
        groundDwell += dt
        if (groundDwell > 4000 && textPlatforms.length > 0) {
          groundDwell = 0; plan() // force new action if stuck at bottom
        }
      } else { groundDwell = 0 }

      if (hipX < -50 || hipX > W + 50 || hipY > H + 150 || hipY < -200) {
         initPosition()
         activeTime = 0
      }

      const ny = hipY - SP
      const hdy = ny - NL - HR
      let tar = neutral(hipX, hipY)
      let useGlow = false

      switch (state) {
        case "RUN": {
          vx += (facing * 6.0 - vx) * 0.08 * n // Slower acceleration, smoother momentum glide
          
          const speedFactor = Math.min(1.2, Math.abs(vx) / 5.5)
          // Master running clock 0.0 to 1.0 continuously looping
          const runClock = (legPhase * 0.035) % 1.0
          const pL = runClock
          const pR = (runClock + 0.5) % 1.0

          // True Fancy Pants locomotion parameters (Fluidity Override)
          const stride = 34 * speedFactor   // Stretching stride
          const lift = 22 * speedFactor     // High knee kick back
          const bob = Math.cos(runClock * Math.PI * 4) * 4.5 * speedFactor // Smooth bouncing
          
          const tilt = Math.max(0, Math.min(1.0, Math.abs(vx) / 5.0))
          const lean = 0.65 * facing * tilt // Smooth atmospheric tilt pushing momentum

          const neck = { x: hipX+Math.sin(lean)*SP, y: hipY-Math.cos(lean)*SP+bob }
          tar.hip = { x:hipX, y:hipY+bob+5 }; tar.neck = neck
          tar.head = { x:neck.x+Math.sin(lean)*(NL+HR), y:neck.y-Math.cos(lean)*(NL+HR) }
          tar.sL = {x:neck.x-6,y:neck.y}; tar.sR = {x:neck.x+6,y:neck.y}

          // Left Foot continuous formula
          const fLx = hipX + Math.cos(pL * Math.PI * 2) * stride * facing
          const fLy = groundY - (pL > 0.5 ? Math.sin((pL - 0.5) * 2 * Math.PI) * lift : 0)
          
          // Right Foot continuous formula
          const fRx = hipX + Math.cos(pR * Math.PI * 2) * stride * facing
          const fRy = groundY - (pR > 0.5 ? Math.sin((pR - 0.5) * 2 * Math.PI) * lift : 0)

          tar.kL = ik(tar.hip.x, tar.hip.y, fLx, fLy, UL, LL, -facing); tar.fL = {x:fLx, y:fLy}
          tar.kR = ik(tar.hip.x, tar.hip.y, fRx, fRy, UL, LL, -facing); tar.fR = {x:fRx, y:fRy}

          // Opposite sweeping arms
          const armReach = 26 * speedFactor
          const hLx = neck.x + Math.cos(pR * Math.PI * 2) * armReach * facing
          const hLy = neck.y + 12 + Math.abs(Math.sin(pR * Math.PI * 2)) * 8
          
          const hRx = neck.x + Math.cos(pL * Math.PI * 2) * armReach * facing
          const hRy = neck.y + 12 + Math.abs(Math.sin(pL * Math.PI * 2)) * 8

          tar.eL = ik(tar.sL.x, tar.sL.y, hLx, hLy, UA, LA, facing); tar.hL = {x:hLx, y:hLy}
          tar.eR = ik(tar.sR.x, tar.sR.y, hRx, hRy, UA, LA, facing); tar.hR = {x:hRx, y:hRy}

          if (runClock < 0.05 || (runClock > 0.5 && runClock < 0.55)) {
             spawnDust(hipX, groundY, 0.4)
          }
          
          if (timer > actionDuration) {
            const r = Math.random()
            if (r < 0.15) go("CATCHBREATH")
            else if (r < 0.30) go("WAVE")
            else plan()
          }
          break
        }

        case "WALK": {
          vx += (facing * 2.8 - vx) * 0.1 * n
          const wLen = WALK_GAIT.length
          const raw = (legPhase * 0.5) % wLen
          const i0 = Math.floor(raw) % wLen
          const i1 = (i0+1) % wLen
          const f = raw - Math.floor(raw)
          const g0 = WALK_GAIT[i0]
          const g1 = WALK_GAIT[i1]
          const lL = lerp(g0.lL,g1.lL,f)*Math.PI/180*facing
          const rL = lerp(g0.rL,g1.rL,f)*Math.PI/180*facing
          const lA = lerp(g0.lA,g1.lA,f)*Math.PI/180*facing
          const rA = lerp(g0.rA,g1.rA,f)*Math.PI/180*facing
          const bob = Math.sin(timer*0.026)*1.5

          tar.hip={x:hipX,y:hipY+bob}; tar.neck={x:hipX,y:ny+bob}; tar.head={x:hipX,y:hdy+bob}
          tar.sL={x:hipX-6,y:ny+bob}; tar.sR={x:hipX+6,y:ny+bob}

          const fLt={x:hipX+Math.sin(lL)*18,y:groundY}
          const fRt={x:hipX+Math.sin(rL)*18,y:groundY}
          tar.kL=ik(hipX,hipY+bob,fLt.x,fLt.y,UL,LL,-facing); tar.fL=fLt
          tar.kR=ik(hipX,hipY+bob,fRt.x,fRt.y,UL,LL,-facing); tar.fR=fRt

          const hLt={x:hipX+Math.sin(lA)*(UA+LA*0.8),y:ny+bob+Math.cos(lA)*(UA+LA*0.8)}
          const hRt={x:hipX+Math.sin(rA)*(UA+LA*0.8),y:ny+bob+Math.cos(rA)*(UA+LA*0.8)}
          tar.eL=ik(hipX-6,ny+bob,hLt.x,hLt.y,UA,LA, facing); tar.hL=hLt
          tar.eR=ik(hipX+6,ny+bob,hRt.x,hRt.y,UA,LA, facing); tar.hR=hRt

          if (timer > actionDuration) plan()
          break
        }

        case "IDLE": {
          vx = 0
          const br = Math.sin(timer*0.003)*1.2
          tar = neutral(hipX, hipY)
          tar.head.y += br; tar.neck.y += br
          if (timer > actionDuration) plan()
          break
        }

        case "SIT": {
          vx *= Math.pow(0.5, n) // rapid stop
          tar = neutral(hipX, hipY)
          
          // Drop hips deep to the ground
          tar.hip = { x: hipX, y: groundY - 14 }
          const tilt = 20 * Math.PI / 180
          tar.neck = { x: hipX + facing*Math.sin(tilt)*SP, y: tar.hip.y - Math.cos(tilt)*SP } // Hunched over
          tar.head = { x: tar.neck.x + facing*4, y: tar.neck.y - NL - HR + 2 }
          
          // Cross-legged lap tracking ground
          tar.kL = { x: hipX + facing*10, y: groundY - 8 }; tar.fL = { x: hipX - facing*4, y: groundY - 2 }
          tar.kR = { x: hipX - facing*6,  y: groundY - 6 }; tar.fR = { x: hipX + facing*12, y: groundY - 2 }
          
          tar.sL = { x: tar.neck.x - 5, y: tar.neck.y }
          tar.sR = { x: tar.neck.x + 5, y: tar.neck.y }
          
          // Right arm holds laptop, left arm types rapidly
          const laptopCenter = { x: hipX + facing*14, y: groundY - 14 }
          
          tar.hR = laptopCenter
          tar.eR = ik(tar.sR.x, tar.sR.y, tar.hR.x, tar.hR.y, UA, LA, facing)
          
          // Typing left arm (insane rapid jitter oscillation based on global Date.now)
          const tNow = Date.now()
          const typingY = laptopCenter.y - 4 + Math.sin(tNow * 0.05) * 4
          const typingX = laptopCenter.x - facing * 4 + Math.cos(tNow * 0.07) * 5
          tar.hL = { x: typingX, y: typingY }
          tar.eL = ik(tar.sL.x, tar.sL.y, tar.hL.x, tar.hL.y, UA, LA, facing)

          if (timer > actionDuration) plan()
          break
        }

        case "CATCHBREATH": {
          vx *= 0.5
          const br=Math.sin(timer*0.012)*2, bend=20*Math.PI/180
          tar.hip={x:hipX,y:hipY}
          tar.neck={x:hipX+facing*Math.sin(bend)*SP, y:hipY-Math.cos(bend)*SP+br}
          tar.head={x:tar.neck.x+facing*5,y:tar.neck.y-NL-HR+br}
          tar.sL={x:tar.neck.x-5,y:tar.neck.y}; tar.sR={x:tar.neck.x+5,y:tar.neck.y}
          const ky=hipY+UL*0.3
          tar.hL={x:hipX+facing*6-6,y:ky}; tar.hR={x:hipX+facing*6+6,y:ky}
          tar.eL=ik(tar.sL.x,tar.sL.y,tar.hL.x,tar.hL.y,UA,LA, facing)
          tar.eR=ik(tar.sR.x,tar.sR.y,tar.hR.x,tar.hR.y,UA,LA, facing)
          tar.kL={x:hipX-4,y:ky+10}; tar.fL={x:hipX-7,y:groundY}
          tar.kR={x:hipX+4,y:ky+10}; tar.fR={x:hipX+7,y:groundY}
          
          if (timer%(actionDuration*0.8) < 100) { if(lookWaitTime<=0) lookWaitTime=400 }
          if (lookWaitTime > 0) {
            tar.head.x -= facing*4 
            tar.head.y -= 3
          }
          if (timer > actionDuration) plan()
          break
        }

        case "WAVE": {
          vx *= 0.8
          const br = Math.sin(timer*0.005)*1.5
          tar = neutral(hipX, hipY)
          tar.head.y += br; tar.neck.y += br
          
          // Friendly rapid wave
          const wag = Math.sin(timer*0.02) * 15
          tar.hR = {x: tar.sR.x + facing*15, y: tar.sR.y - 15 - wag}
          tar.eR = ik(tar.sR.x, tar.sR.y, tar.hR.x, tar.hR.y, UA, LA, facing)
          
          if (timer > actionDuration) plan()
          break
        }

        case "JUMP": {
          const pr = Math.min(1, timer/actionDuration)
          const tuck = Math.sin(pr * Math.PI) // Peak tuck mid-jump
          tar = neutral(hipX, hipY)
          
          tar.kL = {x:hipX-facing*8, y:hipY-tuck*18}; tar.fL = {x:hipX-facing*10, y:hipY+10-tuck*35}
          tar.kR = {x:hipX+facing*8, y:hipY-tuck*15}; tar.fR = {x:hipX+facing*15, y:hipY+15-tuck*30}
          
          tar.hL = {x:tar.sL.x-facing*15, y:tar.sL.y - UA - LA + 8}; tar.eL=ik(tar.sL.x,tar.sL.y,tar.hL.x,tar.hL.y,UA,LA, -facing)
          tar.hR = {x:tar.sR.x+facing*15, y:tar.sR.y - UA - LA + 8}; tar.eR=ik(tar.sR.x,tar.sR.y,tar.hR.x,tar.hR.y,UA,LA, facing)

          if (vy > 0 && Math.random() < 0.05) go("FALL")
          break
        }

        case "PRECISION_JUMP": {
          tar.hip={x:hipX,y:hipY}; tar.neck={x:hipX+facing*4,y:ny+2}; tar.head={x:hipX+facing*6,y:hdy+2}
          tar.sL={x:tar.neck.x-6,y:tar.neck.y}; tar.sR={x:tar.neck.x+6,y:tar.neck.y}
          tar.eL={x:hipX-12,y:ny+UA*0.5}; tar.hL={x:hipX-16,y:ny+UA+LA*0.2}
          tar.eR={x:hipX+12,y:ny+UA*0.5}; tar.hR={x:hipX+16,y:ny+UA+LA*0.2}
          // Legs tucked
          tar.kL={x:hipX+facing*8,y:hipY+UL*0.5}; tar.fL={x:hipX+facing*5,y:hipY+UL*0.5+LL*0.8}
          tar.kR={x:hipX-facing*2,y:hipY+UL*0.7}; tar.fR={x:hipX-facing*2,y:hipY+UL*0.7+LL*0.8}
          if (vy > 2) go("FALL")
          break
        }

        case "FALL": {
           tar.neck = {x:hipX+facing*12, y:ny}
           tar.hL = {x:tar.sL.x-facing*20, y:tar.sL.y - UA - LA*0.8}; tar.eL=ik(tar.sL.x,tar.sL.y,tar.hL.x,tar.hL.y,UA,LA, facing)
           tar.hR = {x:tar.sR.x-facing*10, y:tar.sR.y - UA - LA*0.9}; tar.eR=ik(tar.sR.x,tar.sR.y,tar.hR.x,tar.hR.y,UA,LA, facing)
           
           const stretch = Math.min(1, Math.max(0, vy/15))
           tar.fL = {x:hipX+facing*20*stretch, y:hipY+(UL+LL)*(1-stretch*0.3)}; tar.kL=ik(tar.hip.x,tar.hip.y,tar.fL.x,tar.fL.y,UL,LL,-facing)
           tar.fR = {x:hipX-facing*25*stretch, y:hipY+(UL+LL)*(1-stretch*0.5)}; tar.kR=ik(tar.hip.x,tar.hip.y,tar.fR.x,tar.fR.y,UL,LL,-facing)
           break
        }

        case "LAND": {
          const pr = Math.min(1, timer/250)
          const crouch = Math.sin(pr*Math.PI)
          const drop = crouch * 12
          const bend = crouch * 0.4
          tar.hip={x:hipX,y:hipY+drop}
          tar.neck={x:hipX+facing*Math.sin(bend)*SP, y:hipY+drop-Math.cos(bend)*SP}
          tar.head={x:tar.neck.x+facing*3,y:tar.neck.y-NL-HR}
          tar.sL={x:tar.neck.x-6,y:tar.neck.y}; tar.sR={x:tar.neck.x+6,y:tar.neck.y}
          
          tar.eL={x:hipX-10,y:tar.neck.y+8}; tar.hL={x:hipX-14,y:tar.neck.y+16+crouch*5}
          tar.eR={x:hipX+10,y:tar.neck.y+8}; tar.hR={x:hipX+14,y:tar.neck.y+16+crouch*5}
          
          tar.kL={x:hipX+facing*12+4,y:hipY+drop+UL*0.6}; tar.fL={x:hipX+facing*4,y:groundY}
          tar.kR={x:hipX-facing*8-4,y:hipY+drop+UL*0.6}; tar.fR={x:hipX-facing*8,y:groundY}
          if (pr >= 1) plan()
          break
        }

        case "SLIDE": {
          useGlow = true
          vx *= 0.96 // Friction degrades velocity
          
          // Smooth transition into the slide (takes 150ms to fully crouch)
          const pr = Math.min(1, timer / 150)
          const drop = pr * 16
          
          tar.hip = {x:hipX, y:groundY - 24 + drop} // Hip drops down smoothly
          
          // Spine leans backwards by 50 degrees
          const ang = 50 * Math.PI / 180
          tar.neck = {x: tar.hip.x - facing*Math.sin(ang)*SP, y: tar.hip.y - Math.cos(ang)*SP}
          tar.head = {x: tar.neck.x - facing*4, y: tar.neck.y - NL - HR/2}
          tar.sL = {x: tar.neck.x - 4, y: tar.neck.y}; tar.sR = {x: tar.neck.x + 4, y: tar.neck.y}

          // Back arm planted firmly on the ground behind him to stabilize the slide
          const trh = {x: tar.hip.x - facing*22, y: groundY - 2}
          tar.hR = trh; tar.eR = ik(tar.sR.x, tar.sR.y, trh.x, trh.y, UA, LA, facing)

          // Front arm swept entirely up and back for dynamic balance
          tar.hL = {x: tar.neck.x - facing*18, y: tar.neck.y - 15}
          tar.eL = ik(tar.sL.x, tar.sL.y, tar.hL.x, tar.hL.y, UA, LA, -facing)

          // Front sliding leg stretched fully forward along the ground
          const maxReach = (UL + LL) * 0.95
          tar.fL = {x: tar.hip.x + facing*maxReach, y: groundY}
          tar.kL = ik(tar.hip.x, tar.hip.y, tar.fL.x, tar.fL.y, UL, LL, -facing)

          // Back leg tucked beneath the hips, sliding on the knee/shin
          tar.fR = {x: tar.hip.x - facing*12, y: groundY}
          tar.kR = ik(tar.hip.x, tar.hip.y, tar.fR.x, tar.fR.y, UL, LL, -facing)
          
          if (Math.random() < 0.3) spawnDust(tar.fL.x, groundY, 1) // Less chaotic dust
          if (timer > actionDuration || Math.abs(vx) < 1.5) go("RUN") // Exit smoothly when slow
          break
        }

        case "DIVEROLL": {
          useGlow = true
          const pr = Math.min(1, timer/actionDuration)
          if (pr < 0.2) {
             vx = facing * 8; vy = -4; grounded = false
             // leaping forward like superman
             tar.hip={x:hipX,y:hipY}; tar.neck={x:hipX+facing*SP,y:hipY}
             tar.head={x:tar.neck.x+facing*(NL+HR),y:hipY}
             tar.sL={x:tar.neck.x-2,y:tar.neck.y-4}; tar.sR={x:tar.neck.x+2,y:tar.neck.y+4}
             tar.hL={x:tar.neck.x+facing*20,y:hipY-4}; tar.eL=ik(tar.sL.x,tar.sL.y,tar.hL.x,tar.hL.y,UA,LA, facing)
             tar.hR={x:tar.neck.x+facing*20,y:hipY+4}; tar.eR=ik(tar.sR.x,tar.sR.y,tar.hR.x,tar.hR.y,UA,LA, facing)
             tar.fL={x:hipX-facing*25,y:hipY-8}; tar.kL=ik(tar.hip.x,tar.hip.y,tar.fL.x,tar.fL.y,UL,LL,-facing)
             tar.fR={x:hipX-facing*25,y:hipY+8}; tar.kR=ik(tar.hip.x,tar.hip.y,tar.fR.x,tar.fR.y,UL,LL,-facing)
             flipAngle = 0
          } else if (pr < 0.7) {
             // tuck and roll
             const tpr = (pr-0.2)/0.5
             flipAngle = tpr * Math.PI*2 * facing
             tar = neutral(hipX, hipY)
             // tightly tucked
             tar.neck={x:hipX,y:hipY-SP*0.3}; tar.head={x:hipX,y:hipY-SP*0.5}
             tar.sL={x:hipX-2,y:tar.neck.y}; tar.sR={x:hipX+2,y:tar.neck.y}
             tar.hL={x:hipX,y:hipY}; tar.eL={x:hipX-5,y:hipY-10}
             tar.hR={x:hipX,y:hipY}; tar.eR={x:hipX+5,y:hipY-10}
             tar.kL={x:hipX,y:hipY-15}; tar.fL={x:hipX-2,y:hipY}
             tar.kR={x:hipX,y:hipY-15}; tar.fR={x:hipX+2,y:hipY}
          } else {
             // unroll into sprint
             flipAngle = 0
             tar = neutral(hipX, hipY)
             tar.hip={x:hipX,y:hipY+10}
             vx = facing * 7
          }
          if (pr >= 1) { flipAngle = 0; go("RUN") }
          break
        }

        case "VAULT": {
          const pr = timer/actionDuration
          if (!interactPlat) { go("FALL"); break }
          if (pr < 0.3) {
            vx = facing * 4
            // planting hands on interactPlat
            const ptop = interactPlat.top
            tar.hip={x:hipX,y:ptop-5}
            tar.neck={x:hipX+facing*8,y:ptop-SP*0.7}
            tar.head={x:tar.neck.x+facing*4,y:tar.neck.y-NL-HR}
            tar.sL={x:tar.neck.x-6,y:tar.neck.y}; tar.sR={x:tar.neck.x+6,y:tar.neck.y}
            tar.hL={x:hipX+facing*18-4,y:ptop}; tar.hR={x:hipX+facing*18+4,y:ptop}
            tar.eL=ik(tar.sL.x,tar.sL.y,tar.hL.x,tar.hL.y,UA,LA, facing)
            tar.eR=ik(tar.sR.x,tar.sR.y,tar.hR.x,tar.hR.y,UA,LA, facing)
            tar.kL={x:hipX-facing*8,y:ptop+5}; tar.fL={x:hipX-facing*18,y:groundY}
            tar.kR={x:hipX,y:ptop+5}; tar.fR={x:hipX-facing*8,y:groundY}
          } else if (pr < 0.7) {
            vx = facing * 6.5
            grounded = false; vy = -3
            // hips swinging over hands
            tar.hip={x:hipX,y:hipY-15}
            tar.neck={x:hipX-facing*5,y:hipY-15-SP}
            tar.head={x:tar.neck.x,y:tar.neck.y-NL-HR}
            tar.sL={x:tar.neck.x-6,y:tar.neck.y}; tar.sR={x:tar.neck.x+6,y:tar.neck.y}
            tar.hL={x:interactPlat.left+interactPlat.width/2-4,y:interactPlat.top}
            tar.hR={x:interactPlat.left+interactPlat.width/2+4,y:interactPlat.top}
            tar.eL=ik(tar.sL.x,tar.sL.y,tar.hL.x,tar.hL.y,UA,LA, facing)
            tar.eR=ik(tar.sR.x,tar.sR.y,tar.hR.x,tar.hR.y,UA,LA, facing)
            tar.kL={x:hipX+facing*14,y:hipY-15}; tar.fL={x:hipX+facing*28,y:hipY-5}
            tar.kR={x:hipX+facing*10,y:hipY-10}; tar.fR={x:hipX+facing*20,y:hipY}
          } else {
            vx = facing * 5; tar = neutral(hipX, hipY)
          }
          if (pr >= 1) go("RUN")
          break
        }

        case "WALLRUN": {
          if (!interactPlat) { go("FALL"); break }
          useGlow = true; grounded = false
          const edge = facing===1 ? interactPlat.left : interactPlat.right
          hipX = edge - facing*13; hipY -= 4.5*n // running vertically up

          const step = (timer/120)%2
          tar.hip={x:hipX,y:hipY}
          tar.neck={x:hipX+facing*9,y:hipY-SP*0.8}
          tar.head={x:tar.neck.x+facing*3,y:tar.neck.y-NL-HR+2}
          tar.sL={x:tar.neck.x-5,y:tar.neck.y}; tar.sR={x:tar.neck.x+5,y:tar.neck.y}
          const wh={x:edge,y:hipY-SP*0.4}
          const fh={x:hipX-facing*15,y:hipY-SP*1.2}
          if (facing===1) {
            tar.hR=wh; tar.hL=fh; tar.eR=ik(tar.sR.x,tar.sR.y,wh.x,wh.y,UA,LA, 1); tar.eL=ik(tar.sL.x,tar.sL.y,fh.x,fh.y,UA,LA, 1)
          } else {
            tar.hL=wh; tar.hR=fh; tar.eL=ik(tar.sL.x,tar.sL.y,wh.x,wh.y,UA,LA,-1); tar.eR=ik(tar.sR.x,tar.sR.y,fh.x,fh.y,UA,LA,-1)
          }
          const f1Y=hipY+UL+LL*0.5-(step<1?step*20:0)
          const f2Y=hipY+UL+LL-(step>=1?(2-step)*20:0)
          tar.fL={x:edge,y:f1Y}; tar.fR={x:edge,y:f2Y}
          tar.kL=ik(hipX,hipY,tar.fL.x,tar.fL.y,UL,LL,-facing)
          tar.kR=ik(hipX,hipY,tar.fR.x,tar.fR.y,UL,LL,-facing)

          if (timer > actionDuration || hipY < interactPlat.top + 30) {
            vx = -facing * 5.5; vy = -12; facing = -facing; flipAngle = 0; interactPlat = null
            spawnDust(edge, hipY+20, 8); go("BACKFLIP")
          }
          break
        }

        case "TIC_TAC": {
          if (!interactPlat) { go("FALL"); break }
          useGlow = true; grounded = false
          const edge = facing===1 ? interactPlat.left : interactPlat.right
          hipX = edge - facing*12

          tar.hip={x:hipX,y:hipY}; tar.neck={x:hipX-facing*3,y:hipY-SP}
          tar.head={x:tar.neck.x,y:tar.neck.y-NL-HR}
          tar.sL={x:tar.neck.x-5,y:tar.neck.y}; tar.sR={x:tar.neck.x+5,y:tar.neck.y}
          // hands pushing against wall
          tar.hL={x:edge,y:hipY-SP*0.3}; tar.hR={x:edge,y:hipY-SP*0.6}
          tar.eL=ik(tar.sL.x,tar.sL.y,tar.hL.x,tar.hL.y,UA,LA, facing)
          tar.eR=ik(tar.sR.x,tar.sR.y,tar.hR.x,tar.hR.y,UA,LA, facing)
          // one foot planted, one tucked
          tar.fL={x:edge,y:hipY+10}; tar.kL=ik(tar.hip.x,tar.hip.y,tar.fL.x,tar.fL.y,UL,LL,-facing)
          tar.fR={x:hipX-facing*15,y:hipY+UL*0.5}; tar.kR=ik(tar.hip.x,tar.hip.y,tar.fR.x,tar.fR.y,UL,LL,-facing)

          if (timer > 150) {
            vx = -facing * 8; vy = -11; facing = -facing; interactPlat = null
            spawnDust(edge, hipY+10, 5); go("JUMP")
          }
          break
        }

        case "BACKFLIP": {
          useGlow = true
          const pr = Math.min(1, timer/actionDuration)
          flipAngle = pr * Math.PI*2 * -facing // backward rotation
          const tuck = Math.sin(pr*Math.PI), ext = 1-tuck
          tar.hip={x:hipX,y:hipY}; tar.neck={x:hipX,y:hipY-SP*(1-tuck*0.4)}
          tar.head={x:hipX,y:hipY-SP-NL-HR+tuck*20}
          tar.sL={x:hipX-5,y:tar.neck.y}; tar.sR={x:hipX+5,y:tar.neck.y}
          // arms grabbing knees when tucked
          tar.eL={x:hipX-4-ext*10,y:hipY-SP*0.4-ext*5}; tar.hL={x:hipX-8-ext*15,y:hipY-tuck*10-ext*8}
          tar.eR={x:hipX+4+ext*10,y:hipY-SP*0.4-ext*5}; tar.hR={x:hipX+8+ext*15,y:hipY-tuck*10-ext*8}
          tar.kL={x:hipX-6,y:hipY-tuck*18}; tar.kR={x:hipX+6,y:hipY-tuck*18}
          tar.fL={x:hipX-5,y:hipY+(UL+LL)*(1-tuck)}; tar.fR={x:hipX+5,y:hipY+(UL+LL)*(1-tuck)}
          
          if (pr>=1) { flipAngle=0; if(grounded) { spawnDust(hipX,groundY,7); go("LAND") } else go("FALL") }
          break
        }

        case "HANG": {
          if (!hangPlat) { go("FALL"); break }
          grounded=false; vx=0; vy=0
          // Pendulum swing
          const swayAng = Math.cos(timer*0.005) * Math.min(1, timer/300) * 0.4
          
          tar.hL={x:hangBarX-11,y:hangBarY+3}; tar.hR={x:hangBarX+11,y:hangBarY+3}
          const pivotX = hangBarX, pivotY = hangBarY + 3
          
          tar.sL={x:pivotX-8+Math.sin(swayAng)*UA, y:pivotY+Math.cos(swayAng)*UA}
          tar.sR={x:pivotX+8+Math.sin(swayAng)*UA, y:pivotY+Math.cos(swayAng)*UA}
          tar.neck={x:pivotX+Math.sin(swayAng)*(UA), y:pivotY+Math.cos(swayAng)*(UA)}
          tar.eL=ik(tar.sL.x,tar.sL.y,tar.hL.x,tar.hL.y,UA,LA, facing)
          tar.eR=ik(tar.sR.x,tar.sR.y,tar.hR.x,tar.hR.y,UA,LA, facing)

          tar.hip={x:tar.neck.x+Math.sin(swayAng)*SP, y:tar.neck.y+Math.cos(swayAng)*SP}
          tar.head={x:tar.neck.x+swayAng*5, y:tar.neck.y-NL-HR}
          
          const tuck = Math.sin(timer*0.003)*5
          tar.kL={x:tar.hip.x-6+swayAng*15, y:tar.hip.y+UL-tuck}
          tar.kR={x:tar.hip.x+6+swayAng*15, y:tar.hip.y+UL-tuck*0.5}
          tar.fL={x:tar.hip.x-6+swayAng*30, y:tar.hip.y+UL+LL-tuck*2}
          tar.fR={x:tar.hip.x+6+swayAng*30, y:tar.hip.y+UL+LL-tuck}

          hipX = tar.hip.x; hipY = tar.hip.y

          if (timer > actionDuration) {
             // swinging dismount
             hangCooldown = 2000
             hangPlat = null
             grounded = false
             // boost out based on the sway
             vx = facing * 6 + swayAng * 10
             vy = -10
             go(Math.random() < 0.5 ? "BACKFLIP" : "JUMP")
          }
          break
        }

        case "CURSOR_CHASE": {
          const dx=mouseX-hipX, dy=mouseY-hipY, dist=Math.hypot(dx,dy)
          facing = dx>0 ? 1 : -1; vx = facing*Math.min(8,dist*0.18)
          const lean=0.3*facing
          tar.hip={x:hipX,y:hipY}
          tar.neck={x:hipX+Math.sin(lean)*SP,y:hipY-Math.cos(lean)*SP}
          tar.head={x:tar.neck.x+Math.sin(lean)*(NL+HR),y:tar.neck.y-Math.cos(lean)*(NL+HR)}
          tar.sL={x:tar.neck.x-5,y:tar.neck.y}; tar.sR={x:tar.neck.x+5,y:tar.neck.y}
          
          const maxA = UA + LA - 0.5
          let pL = limitReach(tar.sL.x,tar.sL.y,mouseX*0.4+hipX*0.6-6,mouseY*0.3+ny*0.7,maxA)
          let pR = limitReach(tar.sR.x,tar.sR.y,mouseX*0.4+hipX*0.6+6,mouseY*0.3+ny*0.7,maxA)
          
          tar.hL=pL; tar.eL=ik(tar.sL.x,tar.sL.y,pL.x,pL.y,UA,LA, facing)
          tar.hR=pR; tar.eR=ik(tar.sR.x,tar.sR.y,pR.x,pR.y,UA,LA, facing)

          const gLen = GAIT.length; const raw=(timer/45)%gLen
          const ii=Math.floor(raw)%gLen,jj=(ii+1)%gLen,ff=raw-Math.floor(raw)
          const gg0=GAIT[ii], gg1=GAIT[jj]
          const lL=lerp(gg0.lL,gg1.lL,ff)*Math.PI/180*facing
          const rL=lerp(gg0.rL,gg1.rL,ff)*Math.PI/180*facing
          const fLt2={x:hipX+Math.sin(lL)*(UL+LL),y:Math.min(groundY,hipY+Math.cos(Math.abs(lL))*(UL+LL))}
          const fRt2={x:hipX+Math.sin(rL)*(UL+LL),y:Math.min(groundY,hipY+Math.cos(Math.abs(rL))*(UL+LL))}
          tar.kL=ik(hipX,hipY,fLt2.x,fLt2.y,UL,LL,-facing); tar.fL=fLt2
          tar.kR=ik(hipX,hipY,fRt2.x,fRt2.y,UL,LL,-facing); tar.fR=fRt2

          if (dist < 35) { spawnStars(mouseX,mouseY); go("CURSOR_HELD") }
          if (dist > 300 || timer > 3000) { cursorCooldown=8000; plan() }
          break
        }

        case "CURSOR_HELD": {
          grounded=false; vx=0; vy=0; flipAngle=0
          hipX = lerp(hipX, mouseX, 0.2); hipY = lerp(hipY, mouseY - 10, 0.2)
          const sway=Math.sin(timer*0.003)*8
          tar.hip={x:hipX+sway,y:hipY}; tar.neck={x:hipX+sway*0.6,y:hipY-SP}
          tar.head={x:tar.neck.x+sway*0.3,y:tar.neck.y-NL-HR}
          tar.sL={x:hipX-6+sway*0.4,y:hipY-SP}; tar.sR={x:hipX+6+sway*0.4,y:hipY-SP}

          const maxA = UA + LA - 0.5
          tar.hL=limitReach(tar.sL.x,tar.sL.y,mouseX-9,mouseY-3,maxA)
          tar.hR=limitReach(tar.sR.x,tar.sR.y,mouseX+9,mouseY-3,maxA)
          tar.eL=ik(tar.sL.x,tar.sL.y,tar.hL.x,tar.hL.y,UA,LA, facing)
          tar.eR=ik(tar.sR.x,tar.sR.y,tar.hR.x,tar.hR.y,UA,LA, facing)

          const ls=Math.sin(timer*0.003)*12
          tar.kL={x:hipX-5+ls*0.6,y:hipY+UL}; tar.fL={x:hipX-6+ls,y:hipY+UL+LL}
          tar.kR={x:hipX+5+ls*0.4,y:hipY+UL+5}; tar.fR={x:hipX+6+ls*0.8,y:hipY+UL+LL+6}
          
          ctx.save(); ctx.strokeStyle=RED; ctx.globalAlpha=0.25; ctx.lineWidth=1; ctx.setLineDash([2,4])
          ctx.beginPath(); ctx.moveTo(tar.hL.x,tar.hL.y); ctx.lineTo(mouseX,mouseY); ctx.stroke()
          ctx.beginPath(); ctx.moveTo(tar.hR.x,tar.hR.y); ctx.lineTo(mouseX,mouseY); ctx.stroke()
          ctx.restore()

          if (shakeScore > 50) {
            cursorCooldown=10000; spawnStars(hipX,hipY)
            vx=mouseVX*2+(Math.random()-0.5)*5; vy=mouseVY*2-5
            grounded=false; go("CURSOR_FLUNG")
          }
          break
        }

        case "CURSOR_FLUNG": {
          useGlow=true
          flipAngle += (vx>0?0.25:-0.25)*n
          const fl=Math.sin(timer*0.015)*30*Math.PI/180
          tar.hip={x:hipX,y:hipY}; tar.neck={x:hipX,y:ny}; tar.head={x:hipX,y:hdy}
          tar.sL={x:hipX-6,y:ny}; tar.sR={x:hipX+6,y:ny}
          tar.eL={x:hipX-21,y:ny+fl*UA}; tar.hL={x:hipX-28,y:ny+fl*UA+9}
          tar.eR={x:hipX+21,y:ny-fl*UA}; tar.hR={x:hipX+28,y:ny-fl*UA+9}
          tar.kL={x:hipX-12,y:hipY+UL*0.6}; tar.fL={x:hipX-15,y:hipY+UL+LL*0.45}
          tar.kR={x:hipX+12,y:hipY+UL*0.6}; tar.fR={x:hipX+15,y:hipY+UL+LL*0.45}
          if (grounded) { flipAngle=0; spawnDust(hipX,groundY,7); go("LAND") }
          break
        }

        case "WAVE": {
          vx = 0
          tar = neutral(hipX, hipY)
          tar.hip.y += Math.sin(timer*0.005)*1.5
          tar.head.x += Math.sin(timer*0.005)*1
          tar.neck.x += Math.sin(timer*0.005)*0.5

          // left arm relaxed
          tar.hL = { x: tar.sL.x - 4, y: tar.sL.y + UA + LA - 2 }
          tar.eL = ik(tar.sL.x, tar.sL.y, tar.hL.x, tar.hL.y, UA, LA, facing)

          // right arm waving naturally from the elbow
          const waveAng = Math.sin(timer*0.015) * 35 * Math.PI/180
          tar.eR = { x: tar.sR.x + facing*UA*0.8, y: tar.sR.y + UA*0.4 } // elbow angled to the side
          tar.hR = { x: tar.eR.x + Math.sin(waveAng)*LA, y: tar.eR.y - Math.cos(waveAng)*LA } // hand sweeping side to side

          if (timer > actionDuration) plan()
          break
        }

        default: {
          tar = neutral(hipX, hipY)
          if (timer > 350) plan()
        }
      }

      // Exponential smoothing for heavy, liquid-smooth soft-body physics
      const bf = 1 - Math.pow(0.5, n)
      _curPose = blendPose(_curPose, tar, bf)

      const speed = Math.hypot(vx, vy)
      if (speed > 4 && !["HANG"].includes(state)) {
        trail.unshift(clonePose(_curPose))
        if (trail.length > 5) trail.pop()
      } else if (trail.length > 0) {
        trail.pop()
      }
      const trailA = [0.08, 0.04, 0.02, 0.008, 0.002]
      trail.forEach((tp,i) => { if (i<trailA.length) drawChar(tp, trailA[i]) })

      for (let i=particles.length-1; i>=0; i--) {
        const p=particles[i]
        p.x+=p.vx; p.y+=p.vy; p.vy+=0.15; p.life-=0.04
        if (p.life<=0) { particles.splice(i,1); continue }
        ctx.save(); ctx.globalAlpha=p.life*0.8; ctx.fillStyle=RED; ctx.strokeStyle=RED
        if (p.type==="star") {
          ctx.beginPath(); ctx.arc(p.x,p.y,p.size*p.life,0,Math.PI*2); ctx.fill()
        } else {
          ctx.lineWidth=2; ctx.lineCap="round"
          ctx.beginPath(); ctx.moveTo(p.x,p.y); ctx.lineTo(p.x+p.vx*2.5,p.y+p.vy*2.5); ctx.stroke()
        }
        ctx.restore()
      }

      drawChar(_curPose, 1, useGlow, flipAngle)
      rafId = requestAnimationFrame(tick)
    }

    const onMouseMove = (e: MouseEvent) => { mouseX=e.clientX; mouseY=e.clientY }
    window.addEventListener("mousemove", onMouseMove)

    const onResize = () => {
      W=window.innerWidth; H=window.innerHeight
      canvas.width=W; canvas.height=H
      initialized=false
      extractGeometry()
    }
    window.addEventListener("resize", onResize)

    setTimeout(extractGeometry, 200)
    setTimeout(extractGeometry, 800)
    setTimeout(extractGeometry, 2000)

    const ro = new ResizeObserver(() => setTimeout(extractGeometry, 150))
    ro.observe(document.body)

    rafId = requestAnimationFrame(tick)

    return () => {
      cancelAnimationFrame(rafId)
      window.removeEventListener("resize", onResize)
      window.removeEventListener("mousemove", onMouseMove)
      ro.disconnect()
    }
  }, [])

  return (
    <canvas
      ref={cvs}
      className="fixed inset-0 pointer-events-none z-50"
      style={{ willChange: "transform" }}
    />
  )
}

export default ParkourStickman
