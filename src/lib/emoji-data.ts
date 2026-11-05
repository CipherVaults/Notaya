// A compact emoji list for page icons: bundled in the app, nothing is fetched. Each line is the emoji and
// the words it can be found by. Grouped like the usual pickers.

export interface EmojiEntry {
  e: string;
  k: string;
}
export interface EmojiGroup {
  id: string;
  label: string;
  /** The emoji that stands for the group in the tab strip. */
  glyph: string;
  items: EmojiEntry[];
}

const RAW: Record<string, { label: string; glyph: string; lines: string }> = {
  smileys: {
    label: "Smileys",
    glyph: "😀",
    lines: `
😀 grin happy smile
😃 smile happy joy
😄 laugh happy smile
😁 beam grin teeth
😆 laugh squint
😅 sweat nervous laugh
🤣 rofl laugh floor
😂 tears joy laugh
🙂 slight smile
🙃 upside down silly
😉 wink
😊 blush smile happy
😇 angel halo innocent
🥰 love hearts adore
😍 heart eyes love
🤩 star struck excited wow
😘 kiss blow
😋 yum tasty delicious
😛 tongue silly
😜 wink tongue crazy
🤪 zany wild crazy
🤗 hug warm
🤭 oops giggle hand mouth
🤫 shush quiet secret
🤔 think hmm wonder
🤨 raised eyebrow skeptic
😐 neutral blank
😑 expressionless
😶 silent no mouth
😏 smirk
😒 unamused meh
🙄 eye roll
😬 grimace awkward
😌 relieved calm peace
😔 pensive sad
😪 sleepy tired
😴 sleep zzz tired
😷 mask sick ill
🤒 fever sick thermometer
🤕 hurt bandage injured
🤢 nauseated sick green
🤮 vomit sick
🥵 hot heat sweat
🥶 cold freeze
🥴 woozy dizzy drunk
😵 dizzy knocked out
🤯 mind blown explode
🤠 cowboy hat
🥳 party celebrate birthday
😎 cool sunglasses
🤓 nerd glasses study
🧐 monocle curious inspect
😕 confused
😟 worried
🙁 frown sad
😮 surprised open mouth
😲 astonished shocked
😳 flushed embarrassed
🥺 pleading puppy eyes
😢 cry sad tear
😭 sob crying loud
😱 scream fear shock
😖 confounded
😞 disappointed
😓 downcast sweat
😩 weary tired
😫 tired exhausted
🥱 yawn bored sleepy
😤 triumph steam huff
😡 angry rage red
😠 angry mad
🤬 swearing cursing angry
😈 devil imp smile
👿 imp angry devil
💀 skull dead
☠️ skull crossbones danger
💩 poop
🤡 clown
👻 ghost halloween boo
👽 alien ufo
🤖 robot bot ai
🎃 pumpkin halloween
😺 cat smile
😸 cat grin
😻 cat heart eyes
🙈 see no evil monkey
🙉 hear no evil monkey
🙊 speak no evil monkey
💋 kiss lips
💯 hundred perfect score
💥 boom collision
💫 dizzy stars
💦 sweat drops water
💤 sleep zzz
`,
  },
  people: {
    label: "People",
    glyph: "👋",
    lines: `
👋 wave hello hand
🤚 raised back hand
✋ hand stop high five
🖖 vulcan salute spock
👌 ok perfect hand
🤌 pinched fingers italian
✌️ victory peace
🤞 crossed fingers luck
🤟 love you gesture
🤘 rock horns metal
🤙 call me shaka
👈 point left
👉 point right
👆 point up
👇 point down
☝️ index up one
👍 thumbs up yes good like
👎 thumbs down no bad
✊ fist power
👊 punch fist bump
👏 clap applause bravo
🙌 raised hands celebrate hooray
👐 open hands
🤝 handshake deal agreement
🙏 pray thanks please folded hands
✍️ writing hand pen
💅 nail polish
💪 muscle strong flex bicep
🧠 brain mind think smart
🫀 heart organ anatomical
👀 eyes look see watch
👁️ eye
👅 tongue
👄 mouth lips
👶 baby child
🧒 child kid
👦 boy
👧 girl
🧑 person adult
👨 man
👩 woman
🧓 older person
👴 old man grandfather
👵 old woman grandmother
🧑‍💻 technologist developer coder computer
🧑‍🎓 student graduate school
🧑‍🏫 teacher professor
🧑‍⚕️ doctor health worker
🧑‍🍳 cook chef
🧑‍🎨 artist painter
🧑‍🚀 astronaut space
🧑‍🔬 scientist lab
🧑‍🔧 mechanic
🧑‍💼 office worker business
🦸 superhero
🧙 mage wizard magic
🧚 fairy
🧛 vampire
🧜 merperson mermaid
🥷 ninja
💃 dancer dance woman
🕺 dancer dance man
🚶 walking walk
🏃 running run
🧘 yoga lotus meditate
🛌 bed sleeping
👪 family
💑 couple heart
👣 footprints steps
`,
  },
  nature: {
    label: "Nature",
    glyph: "🌿",
    lines: `
🐶 dog puppy pet
🐱 cat kitten pet
🐭 mouse
🐹 hamster
🐰 rabbit bunny
🦊 fox
🐻 bear
🐼 panda
🐨 koala
🐯 tiger
🦁 lion
🐮 cow
🐷 pig
🐸 frog
🐵 monkey
🐔 chicken
🐧 penguin
🐦 bird
🐤 chick baby
🦆 duck
🦅 eagle
🦉 owl night wise
🦇 bat
🐺 wolf
🐴 horse
🦄 unicorn magic
🐝 bee honey busy
🐛 bug caterpillar
🦋 butterfly
🐌 snail slow
🐞 ladybug beetle
🐜 ant
🪲 beetle
🕷️ spider
🐢 turtle slow
🐍 snake python
🦎 lizard
🐙 octopus
🦑 squid
🦀 crab
🐠 fish tropical
🐟 fish
🐬 dolphin
🐳 whale
🦈 shark
🐊 crocodile
🐘 elephant
🦒 giraffe
🦓 zebra
🐪 camel desert
🦘 kangaroo
🐾 paw prints pets
🌵 cactus desert
🎄 christmas tree
🌲 evergreen tree pine
🌳 tree deciduous
🌴 palm tree beach
🌱 seedling sprout grow
🌿 herb leaf plant
☘️ shamrock clover
🍀 four leaf clover luck
🍁 maple leaf autumn fall
🍂 fallen leaf autumn
🍃 leaf wind
🌷 tulip flower spring
🌹 rose flower love
🌺 hibiscus flower
🌻 sunflower flower
🌼 blossom flower daisy
🌸 cherry blossom flower spring
💐 bouquet flowers
🍄 mushroom
🌾 rice sheaf grain
🪨 rock stone
🌍 earth globe europe africa
🌎 earth globe americas
🌏 earth globe asia
🌙 crescent moon night
🌕 full moon
⭐ star
🌟 glowing star sparkle
✨ sparkles shine new
⚡ lightning bolt energy fast
☄️ comet
🔥 fire hot flame
🌈 rainbow
☀️ sun sunny weather
🌤️ sun cloud weather
⛅ partly cloudy weather
☁️ cloud weather
🌧️ rain weather
⛈️ thunderstorm weather
❄️ snowflake cold winter
☃️ snowman winter
💧 droplet water
🌊 wave ocean sea
`,
  },
  food: {
    label: "Food",
    glyph: "🍎",
    lines: `
🍏 green apple
🍎 apple red fruit
🍐 pear
🍊 orange tangerine fruit
🍋 lemon citrus
🍌 banana
🍉 watermelon
🍇 grapes
🍓 strawberry
🫐 blueberries
🍒 cherries
🍑 peach
🥭 mango
🍍 pineapple
🥥 coconut
🥝 kiwi
🍅 tomato
🍆 eggplant aubergine
🥑 avocado
🥦 broccoli
🥬 leafy green lettuce
🥒 cucumber
🌶️ hot pepper chili spicy
🌽 corn maize
🥕 carrot
🧄 garlic
🧅 onion
🥔 potato
🍠 sweet potato
🥐 croissant
🍞 bread loaf
🥖 baguette
🧀 cheese
🥚 egg
🍳 cooking fried egg breakfast
🥞 pancakes
🧇 waffle
🥓 bacon
🥩 steak meat
🍗 poultry leg chicken
🍖 meat bone
🌭 hot dog
🍔 burger hamburger
🍟 fries
🍕 pizza
🥪 sandwich
🌮 taco
🌯 burrito wrap
🥗 salad healthy
🍝 spaghetti pasta
🍜 ramen noodles soup
🍲 stew pot food
🍛 curry rice
🍣 sushi
🍱 bento box
🥟 dumpling
🍤 shrimp fried
🍙 rice ball onigiri
🍚 cooked rice
🍦 ice cream soft serve
🍨 ice cream
🍩 doughnut donut
🍪 cookie
🎂 birthday cake
🍰 cake slice shortcake
🧁 cupcake
🍫 chocolate bar
🍬 candy sweet
🍭 lollipop
🍯 honey pot
🍿 popcorn movie
☕ coffee tea hot drink
🍵 green tea cup
🧋 bubble tea boba
🥤 cup straw soda drink
🍺 beer mug
🍷 wine glass
🥂 clinking glasses cheers champagne
🍸 cocktail
🧃 juice box beverage
🥛 milk glass
🧊 ice cube
🍴 fork knife cutlery restaurant
🥢 chopsticks
`,
  },
  activity: {
    label: "Activity",
    glyph: "⚽",
    lines: `
⚽ soccer football ball
🏀 basketball
🏈 american football
⚾ baseball
🎾 tennis
🏐 volleyball
🏉 rugby
🎱 pool billiards eight ball
🏓 ping pong table tennis
🏸 badminton
🥊 boxing glove
🥋 martial arts judo karate
⛳ golf flag
🏹 archery bow arrow
🎣 fishing
🤿 diving mask snorkel
⛸️ ice skate
🎿 ski
🏂 snowboard
🏋️ weightlifting gym lift
🚴 cycling bike
🏊 swimming swim
🧗 climbing climber
🏆 trophy winner prize
🥇 gold medal first
🥈 silver medal second
🥉 bronze medal third
🎖️ military medal
🎫 ticket
🎟️ admission tickets
🎪 circus tent
🎭 performing arts theater drama masks
🎨 art palette paint
🎬 clapper board movie film
🎤 microphone sing karaoke
🎧 headphones music listen
🎼 musical score music
🎹 piano keyboard music
🥁 drum music
🎷 saxophone music jazz
🎺 trumpet music
🎸 guitar music rock
🎻 violin music
🎲 dice game random
♟️ chess pawn game
🎯 target bullseye goal dart
🎳 bowling
🎮 video game controller play
🕹️ joystick arcade game
🧩 puzzle piece jigsaw
🪁 kite
🎁 gift present wrapped
🎀 ribbon bow
🎈 balloon party
🎉 party popper celebrate tada
🎊 confetti ball celebrate
🎋 tanabata tree
🎐 wind chime
🧧 red envelope
`,
  },
  travel: {
    label: "Travel",
    glyph: "✈️",
    lines: `
🚗 car automobile
🚕 taxi cab
🚙 suv
🚌 bus
🚎 trolleybus
🏎️ racing car
🚓 police car
🚑 ambulance
🚒 fire engine
🚚 delivery truck
🚜 tractor
🏍️ motorcycle
🛵 scooter
🚲 bicycle bike
🛴 kick scooter
🚂 locomotive train steam
🚆 train
🚇 metro subway
🚊 tram
🚁 helicopter
✈️ airplane plane flight trip
🛫 departure takeoff flight
🛬 arrival landing flight
🚀 rocket launch startup space
🛸 flying saucer ufo
🛰️ satellite
⛵ sailboat
🚢 ship cruise
⚓ anchor
🚧 construction work
⛽ fuel pump gas
🚦 traffic light
🗺️ world map travel
🧭 compass navigation
🗽 statue of liberty new york
🗼 tokyo tower
🏰 castle
🏯 japanese castle
🎡 ferris wheel fair
🎢 roller coaster
⛲ fountain
⛺ tent camping
🏕️ camping
🏖️ beach umbrella holiday vacation
🏝️ desert island
🏜️ desert
🏔️ snow capped mountain
⛰️ mountain
🌋 volcano
🗻 mount fuji
🏠 house home
🏡 house garden home
🏢 office building work
🏥 hospital
🏦 bank
🏨 hotel
🏪 convenience store shop
🏫 school
🏛️ classical building museum
⛪ church
🕌 mosque
🕍 synagogue
🕋 kaaba
🌆 cityscape dusk
🌃 night with stars city
🌉 bridge night
🌅 sunrise
🌄 sunrise mountains
🌇 sunset
🌌 milky way galaxy
`,
  },
  objects: {
    label: "Objects",
    glyph: "💡",
    lines: `
⌚ watch time
📱 mobile phone smartphone
💻 laptop computer
🖥️ desktop computer monitor
⌨️ keyboard
🖱️ computer mouse
💾 floppy disk save
💿 optical disk cd
📷 camera photo
📸 camera flash photo
📹 video camera
📺 television tv
📻 radio
🎙️ studio microphone podcast
⏰ alarm clock time wake
⏳ hourglass time
⌛ hourglass done time
🔋 battery power
🔌 electric plug
💡 light bulb idea
🔦 flashlight torch
🕯️ candle
🧯 fire extinguisher
💰 money bag
💵 dollar banknote cash
💳 credit card payment
💎 gem diamond
⚖️ balance scale law justice
🧰 toolbox tools
🔧 wrench tool fix
🔨 hammer tool build
⚒️ hammer pick tools
🛠️ hammer wrench tools build
⛏️ pick
🔩 nut bolt
⚙️ gear settings cog
🧱 brick wall
🔗 link chain url
⛓️ chains
🧲 magnet
💣 bomb
🛡️ shield protect security
🔮 crystal ball fortune
🧿 nazar amulet
💊 pill medicine
💉 syringe vaccine shot
🩺 stethoscope doctor health
🩹 adhesive bandage
🔬 microscope science
🔭 telescope astronomy
📡 satellite antenna
🧬 dna genetics biology
🧪 test tube chemistry experiment
🧫 petri dish
🧹 broom clean sweep
🧺 basket laundry
🚿 shower
🛁 bathtub bath
🔑 key
🗝️ old key
🚪 door
🛋️ couch lamp sofa
🛏️ bed
🪑 chair
🖼️ framed picture art
🛒 shopping cart
🎒 backpack school bag
👓 glasses
🕶️ sunglasses
👔 necktie shirt work
👕 t-shirt shirt
👖 jeans pants
👗 dress
👑 crown king queen
🎩 top hat
🎓 graduation cap school
💄 lipstick makeup
💍 ring wedding
📚 books library study
📖 open book read
📕 closed book red
📗 green book
📘 blue book
📙 orange book
📓 notebook
📔 notebook decorative cover
📒 ledger
📃 page curl document
📄 page facing up document
📰 newspaper news
🗞️ rolled newspaper
📑 bookmark tabs
🔖 bookmark
🏷️ label tag
📝 memo note pencil write
✏️ pencil write
✒️ black nib pen
🖋️ fountain pen
🖊️ pen
🖌️ paintbrush
🖍️ crayon
📁 folder file
📂 open folder
🗂️ card index dividers
📅 calendar date
📆 tear off calendar
🗒️ spiral notepad notes
🗓️ spiral calendar
📇 card index
📈 chart increasing growth trend
📉 chart decreasing decline
📊 bar chart stats data
📋 clipboard list
📌 pushpin pin
📍 round pushpin location place
📎 paperclip attach
🖇️ linked paperclips
📏 straight ruler
📐 triangular ruler geometry
✂️ scissors cut
🗃️ card file box
🗄️ file cabinet archive
🗑️ wastebasket trash delete
🔒 locked lock secure private
🔓 unlocked open
🔏 lock with pen
🔐 lock with key secure
📦 package box parcel
📫 mailbox mail
📧 email mail envelope
✉️ envelope letter mail
📨 incoming envelope
📤 outbox tray send
📥 inbox tray receive
📣 megaphone announce
📢 loudspeaker announce
🔔 bell notification
🔕 bell off mute
📞 telephone receiver call
☎️ telephone
🔍 magnifying glass search left
🔎 magnifying glass search right
`,
  },
  symbols: {
    label: "Symbols",
    glyph: "❤️",
    lines: `
❤️ red heart love
🧡 orange heart
💛 yellow heart
💚 green heart
💙 blue heart
💜 purple heart
🖤 black heart
🤍 white heart
🤎 brown heart
💔 broken heart
❣️ heart exclamation
💕 two hearts love
💞 revolving hearts
💓 beating heart
💗 growing heart
💖 sparkling heart
💘 heart arrow cupid
💝 heart ribbon gift
☮️ peace
✝️ cross
☯️ yin yang balance
♈ aries zodiac
♉ taurus zodiac
♊ gemini zodiac
♋ cancer zodiac
♌ leo zodiac
♍ virgo zodiac
♎ libra zodiac
♏ scorpio zodiac
♐ sagittarius zodiac
♑ capricorn zodiac
♒ aquarius zodiac
♓ pisces zodiac
✅ check mark button done complete
☑️ check box done
✔️ check mark done
❌ cross mark no wrong
❎ cross mark button
➕ plus add
➖ minus subtract
➗ divide
✖️ multiply times
❓ question mark red
❔ white question mark
❗ exclamation mark red important
❕ white exclamation mark
‼️ double exclamation
⁉️ exclamation question
⚠️ warning caution
🚫 prohibited forbidden no
⛔ no entry stop
🔴 red circle
🟠 orange circle
🟡 yellow circle
🟢 green circle
🔵 blue circle
🟣 purple circle
⚫ black circle
⚪ white circle
🟥 red square
🟧 orange square
🟨 yellow square
🟩 green square
🟦 blue square
🟪 purple square
⬛ black square
⬜ white square
🔶 large orange diamond
🔷 large blue diamond
🔸 small orange diamond
🔹 small blue diamond
🔺 red triangle up
🔻 red triangle down
💠 diamond with dot
🔘 radio button
▶️ play button
⏸️ pause button
⏹️ stop button
⏺️ record button
⏭️ next track
⏮️ last track
🔀 shuffle
🔁 repeat loop
🔄 counterclockwise arrows refresh sync
⬆️ up arrow
⬇️ down arrow
⬅️ left arrow
➡️ right arrow
↗️ up right arrow
↘️ down right arrow
↩️ return left arrow back
↪️ right arrow curving
🔙 back arrow
🔚 end arrow
🔛 on arrow
🔜 soon arrow
🔝 top arrow
♻️ recycling recycle
⚜️ fleur de lis
🔱 trident emblem
📛 name badge
⭕ hollow red circle
🆒 cool button
🆕 new button
🆓 free button
🆗 ok button
🆙 up button
1️⃣ keycap one
2️⃣ keycap two
3️⃣ keycap three
4️⃣ keycap four
5️⃣ keycap five
💲 heavy dollar sign money
💱 currency exchange
™️ trade mark
©️ copyright
®️ registered
〰️ wavy dash
➰ curly loop
🏁 chequered flag finish race
🚩 triangular flag
🏳️ white flag
🏴 black flag
🏳️‍🌈 rainbow flag pride
🏴‍☠️ pirate flag
`,
  },
};

function parse(lines: string): EmojiEntry[] {
  return lines
    .split("\n")
    .map((l) => l.trim())
    .filter(Boolean)
    .map((l) => {
      const i = l.indexOf(" ");
      return { e: l.slice(0, i), k: l.slice(i + 1) };
    });
}

export const EMOJI_GROUPS: EmojiGroup[] = Object.entries(RAW).map(([id, g]) => ({ id, label: g.label, glyph: g.glyph, items: parse(g.lines) }));

export const ALL_EMOJI: EmojiEntry[] = EMOJI_GROUPS.flatMap((g) => g.items);

/** Emoji whose name words contain every word typed (or that are the emoji typed). */
export function searchEmoji(query: string): EmojiEntry[] {
  const words = query.toLowerCase().split(/\s+/).filter(Boolean);
  if (!words.length) return [];
  return ALL_EMOJI.filter((x) => words.every((w) => x.k.includes(w) || x.e === w));
}

export function randomEmoji(): string {
  return ALL_EMOJI[Math.floor(Math.random() * ALL_EMOJI.length)].e;
}
