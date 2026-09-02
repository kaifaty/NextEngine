"""NGQ7 wall-monolayer gates over `nonlocal-feasibility --game-surface-stream`
dumps (`--dump-particles`) and the streamed closing surface of the same run.

Usage: python3 lab/scripts/nonlocal_wall_monolayer_gates.py <run-dir>...
Each run directory holds `surface.bin` and `p-cycle0-step<N>.bin` files and
is named `<lane>-L<layers>[support]` (for example `16k-L2density`).
The gate definitions are frozen in
docs/plans/nonlocal-gpu-full-step-performance/21-wall-monolayer-boundary-support.md.
"""
import glob
import os
import re
import struct
import sys
def frames(path):
    data=open(path,'rb').read(); pos=0; out={}
    while pos < len(data):
        step=struct.unpack_from('<i',data,pos+8)[0]; nv,nt=struct.unpack_from('<QQ',data,pos+64)
        verts=struct.unpack_from('<%dd'%(nv*3),data,pos+104); out[step]=(nv,verts); pos+=104+nv*24+nt*12
    return out
def dump(path):
    d=open(path,'rb').read(); n=struct.unpack_from('<Q',d,0)[0]; v=struct.unpack_from('<%df'%(3*n),d,8)
    return [(v[3*i],v[3*i+1],v[3*i+2]) for i in range(n)]
def analyse(directory, wall_x, depth_z, dam_x, label):
    surf=frames(directory+'/surface.bin')
    steps=sorted(int(re.search(r'step(\d+)',p).group(1)) for p in glob.glob(directory+'/p-cycle0-step*.bin'))
    dt=4/240; band_lo=wall_x-0.3; capacity=0.3*depth_z/0.05**2
    g1=0.0; g1_step=None; stall_run=0; stall_max=0; stall_start=None; g2=None; front_step=None
    prev=None
    for step in steps:
        cur=dump(directory+f'/p-cycle0-step{step}.bin')
        band=[i for i,(x,y,z) in enumerate(cur) if band_lo<=x<wall_x]
        bottom=sum(1 for i in band if cur[i][1]<0.06)
        comp=bottom/capacity
        if comp>g1: g1,g1_step=comp,step
        nv,verts=surf.get(step,(0,()))
        xs=verts[0::3]; ys=verts[1::3]
        wall_h=[y for x,y in zip(xs,ys) if x>=band_lo]
        mean_wall_h=sum(wall_h)/len(wall_h) if wall_h else 0.0
        if front_step is None and xs and max(xs)>=wall_x-0.05: front_step=step
        if g2 is None and front_step is not None:
            best=max(((y,x) for x,y in zip(xs,ys) if x>wall_x/2), default=(0,0))
            if best[0]>0.25: g2=(step, wall_x-best[1], best[0])
        vx=None
        if prev is not None and len(band)>=50:
            vx=sum(cur[i][0]-prev[i][0] for i in band)/len(band)/dt
        stalled = vx is not None and vx<0.1 and mean_wall_h<0.1
        if stalled:
            stall_run+=1
            if stall_run==1: stall_start=step
            if stall_run>stall_max: stall_max=stall_run; stall_peak=(stall_start,step)
        else: stall_run=0
        prev=cur
    front_speed = (wall_x-dam_x)/(front_step*(1/240)) if front_step else float('nan')
    print(f'{label}: front reaches wall at step {front_step} (mean front speed {front_speed:.2f} m/s from x={dam_x})')
    print(f'{label}: G1 compression {g1:.2f} (step {g1_step}) {"PASS" if g1<=1.2 else "FAIL"} | G2 first crest>0.25m at step {g2[0] if g2 else None}, {g2[1]:.2f} m from wall, h {g2[2]:.2f} {"PASS" if g2 and g2[1]<=0.15 else "FAIL"} | G3 longest stall {stall_max} frames {stall_peak if stall_max else ""} {"PASS" if stall_max<=10 else "FAIL"} | front at wall step {front_step}')
lanes={'16k':(4.0,2.0,2.0),'4k':(2.0,1.0,1.0),'48k':(4.0,2.0,4.0)}
for d in sorted(sys.argv[1:] or glob.glob('/tmp/nonlocal-ngq7/*-L*')):
    lane=d.split('/')[-1].split('-L')[0]; layers=d.split('-L')[1]
    if lane=='48k': continue
    if not glob.glob(d+'/p-cycle0-step960.bin') or any(os.path.getsize(f)==0 for f in glob.glob(d+'/p-cycle0-step*.bin')): print('skip incomplete',d); continue
    analyse(d, *lanes[lane], f'{lane} layers {layers}')
