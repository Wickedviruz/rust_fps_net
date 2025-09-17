🔹 Steg 1 – Rörelse och kamera

Just nu kan du gå runt, men:

Lägg till gravity + hopp (via Rapier).

Gör en karaktärcontroller (så du inte kan flyga genom lådor).

Gör mus-rotationen mer "FPS-kamera" (separat yaw för kropp, pitch för kamera).

🔹 Steg 2 – Skjutmekanik

Implementera hitscan på klienten (redan delvis där).

Låt servern validera träffar (server-authoritativ combat).

Lägg till projektiler (för raketer/sniper), bara som övning.

🔹 Steg 3 – Multiplayer grund

Just nu kan klienten skicka input och få en position tillbaka.

Vi kan:

Lägga till flera spelare.

Låta servern spawn:a entiteter för varje spelare.

Klienten ser andra spelare röra sig.

🔹 Steg 4 – Game loop / enkel match

Respawn när man dör.

Enkel scoreboard i UI.

Win condition ("först till 10 kills").

🔹 Steg 5 – Grafik & polish

Lägg till riktiga modeller (importera .glb i Bevy).

Enkla shaders, vapenmodell, kanske HUD.