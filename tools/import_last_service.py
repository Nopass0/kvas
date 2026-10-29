"""Copies the LAST SERVICE source assets from .test_assets/horror_day into games/last_service/assets.

The third-party packs keep their own licences and are not committed (see .gitignore); run this once
after cloning to make the game playable:  python tools/import_last_service.py
"""
import os
import shutil
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SRC = os.path.join(ROOT, '.test_assets', 'horror_day')
PROD = os.path.join(SRC, '_production')
DST = os.path.join(ROOT, 'games', 'last_service', 'assets')


def copy(src, dst):
    s = os.path.join(SRC, src) if not os.path.isabs(src) else src
    d = os.path.join(DST, dst)
    if not os.path.exists(s):
        print('missing', s)
        return
    if os.path.isdir(s):
        shutil.copytree(s, d, dirs_exist_ok=True)
    else:
        os.makedirs(os.path.dirname(d), exist_ok=True)
        shutil.copy2(s, d)


def main():
    if not os.path.isdir(SRC):
        sys.exit('expected the horror_day pack in ' + SRC)
    # World, grass, sky, materials made for the game.
    copy(os.path.join(PROD, 'v3', 'scenes', 'last_service_world.glb'), 'world/last_service_world.glb')
    copy(os.path.join(PROD, 'v3', 'grass'), 'grass')
    copy(os.path.join(PROD, 'v3', 'backgrounds'), 'backgrounds')
    copy(os.path.join(PROD, 'v3', 'shaders'), 'shaders/ls')
    # Interface: logo, pixel font, icons, buttons.
    copy(os.path.join(PROD, 'v2', 'branding'), 'ui/branding')
    copy(os.path.join(PROD, 'v2', 'ui', 'font'), 'ui/font')
    copy(os.path.join(PROD, 'v2', 'ui', 'icons'), 'ui/icons')
    copy(os.path.join(PROD, 'v2', 'decals'), 'decals')
    copy(os.path.join(PROD, 'generated', 'notes'), 'ui/notes')
    # Props: the production's own models (lockers, benches, the bus stop...) and the packs.
    copy(os.path.join(PROD, 'v3', 'models'), 'models')
    copy('psx-horror-props-free/glb', 'models/props')
    copy('psx-horror-props-free/textures', 'models/props/textures')
    copy('PSX Kitchen Props Pack/glb', 'models/kitchen')
    copy('PSX Kitchen Props Pack/textures', 'models/kitchen/textures')
    copy('Models', 'models/misc')
    copy('Textures', 'models/misc/textures')
    copy('Textures', 'models/textures')
    copy('glb', 'models/knives')
    # Characters.
    for f in ['MonsterPSX.fbx', 'MonsterPSX_Diffuse.png', 'female_secretary.fbx', 'secretary_tex.png', 'male_casual.fbx', 'man_tex.png']:
        copy(f, 'models/characters/' + f)
    # Sound: the game's own synthesized effects and ambience, plus music.
    copy(os.path.join(PROD, 'audio'), 'audio/sfx')
    for name in ['Dark Forest', 'Deep', 'Expecting', 'Lost', 'Mystery', 'Over The Radio', 'Chase 2', 'Doom Approaches', 'Roll Credits', 'Nostalgic', 'Jumpscare', 'Stranger']:
        copy(name + '.m4a', 'audio/music/' + name.lower().replace(' ', '_') + '.m4a')
    # The world mesh has posed figures and props the game places itself: take them out.
    import subprocess
    world = os.path.join(DST, 'world', 'last_service_world.glb')
    subprocess.run([sys.executable, os.path.join(ROOT, 'tools', 'clean_world_glb.py'), world], check=True)
    # ...then cut it into separate objects (zones, materials, single pieces of furniture). The
    # merged file is not needed by the game afterwards and would only double the package.
    subprocess.run([sys.executable, os.path.join(ROOT, 'tools', 'split_world_glb.py'), world], check=True)
    os.remove(world)
    # Door, fan and grass sounds the production did not have.
    subprocess.run([sys.executable, os.path.join(ROOT, 'tools', 'make_last_service_sfx.py')], check=True)
    print('assets ->', DST)
    print('next: cargo run -p kvas_render --example make_item_icons (inventory icons from the models)')


if __name__ == '__main__':
    main()
