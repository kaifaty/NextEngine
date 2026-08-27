use super::{AudioFormat, FrozenObject, FrozenRecording};

macro_rules! recording {
    ($condition:literal, $clip:literal, $file:literal, $bytes:literal, $sha:literal) => {
        FrozenRecording {
            condition_id: $condition,
            clip_id: $clip,
            osf_file_id: $file,
            source_file_name: concat!("Clip ", $clip, ".ogx"),
            expected_bytes: $bytes,
            sha256: $sha,
            audio_format: AudioFormat::OggVorbis,
        }
    };
}

pub(super) const OBJECTS: &[FrozenObject] = &[
    FrozenObject {
        source_id: "ycb-impact-vertical-object-005-tomato-soup-can",
        object_id: "5",
        object_name: "Tomato soup can",
        primary_material_label: "Aluminium",
        secondary_material_label: None,
        source_split: "vertical-known",
        recordings: &[
            recording!(
                "vertical-known-005",
                "1",
                "622b264822fc5e016d1f0fb9",
                350764,
                "4b7411b4033a73d743ea88fe5c469f114191397e95919dda6db4310de8686097"
            ),
            recording!(
                "vertical-known-005",
                "2",
                "622b264753a4e80169515c71",
                320732,
                "339ae31381a1c043772fa12a46ad38c4a9df05ecea798490b3424740e8109187"
            ),
        ],
    },
    FrozenObject {
        source_id: "ycb-impact-vertical-object-007-tune-fish-can",
        object_id: "7",
        object_name: "Tune fish can",
        primary_material_label: "Aluminium",
        secondary_material_label: None,
        source_split: "vertical-known",
        recordings: &[
            recording!(
                "vertical-known-007",
                "1",
                "622b269946bf10015e593355",
                339113,
                "d427dae607aaeb3307686fafe206dd7361103b97fad1978e2827da6bcd36ab2f"
            ),
            recording!(
                "vertical-known-007",
                "2",
                "622b269953a4e8015e5173a7",
                349653,
                "20a668804e91966bda48fd33dfabb56e6df0b20b175deb3d0be234db8cced00d"
            ),
        ],
    },
    FrozenObject {
        source_id: "ycb-impact-vertical-object-002-master-chef-can",
        object_id: "2",
        object_name: "Master chef can",
        primary_material_label: "Aluminium",
        secondary_material_label: Some("Paper"),
        source_split: "vertical-unknown",
        recordings: &[
            recording!(
                "vertical-unknown-002",
                "1",
                "622b682622fc5e024c1f1450",
                448549,
                "b8dac6656cc9de6028ff672232560f1e8da507414585abc155a4a4434eb7a831"
            ),
            recording!(
                "vertical-unknown-002",
                "2",
                "622b682646bf100226593651",
                475153,
                "4dcc63f3c831ea894d885c6b2f1c0ec69ca3101caa10b55f7d18a47989378207"
            ),
        ],
    },
    FrozenObject {
        source_id: "ycb-impact-vertical-object-008-pudding-box",
        object_id: "8",
        object_name: "Pudding box",
        primary_material_label: "Paper",
        secondary_material_label: None,
        source_split: "vertical-known",
        recordings: &[
            recording!(
                "vertical-known-008",
                "1",
                "622b26b753a4e80169515de8",
                292153,
                "80cb6ef93e0b31a562c35fba3951eb3cbcd66bbd6dcf04137e9f638af05a4adf"
            ),
            recording!(
                "vertical-known-008",
                "2",
                "622b26b653a4e8016a51601e",
                303335,
                "a73f296d01ebae87dc6d2a02b61fd0c347b4abfe4b8b7dab7e3de66c1c4c3a98"
            ),
        ],
    },
    FrozenObject {
        source_id: "ycb-impact-vertical-object-003-cracker-box",
        object_id: "3",
        object_name: "Cracker box",
        primary_material_label: "Paper",
        secondary_material_label: None,
        source_split: "vertical-known",
        recordings: &[
            recording!(
                "vertical-known-003",
                "1",
                "622b260746bf100163593631",
                314090,
                "b29a2c9f3926cb1ca872b9f6cf3bd8a85aae0125ec3990e760da81ee04470a5e"
            ),
            recording!(
                "vertical-known-003",
                "2",
                "622b260822fc5e01711f11c1",
                446807,
                "bf1d468bf172a7ffb54b2b9119f7e6d55106404d52d68454c22d081f281246dd"
            ),
        ],
    },
    FrozenObject {
        source_id: "ycb-impact-vertical-object-004-sugar-box",
        object_id: "4",
        object_name: "Sugar box",
        primary_material_label: "Paper",
        secondary_material_label: None,
        source_split: "vertical-known",
        recordings: &[
            recording!(
                "vertical-known-004",
                "1",
                "622b262953a4e80169515c0f",
                334086,
                "41a6dd873b9f6cb21abedf6051eea695319d3a1923516638a7f377395566ebd8"
            ),
            recording!(
                "vertical-known-004",
                "2",
                "622b262946bf1001635936eb",
                365105,
                "f6e7908ea6a09fd56aa78a2b9494d24bad3784df56b67c9e1b475f00ba6def8f"
            ),
        ],
    },
    FrozenObject {
        source_id: "ycb-impact-vertical-object-024-bowl",
        object_id: "24",
        object_name: "Bowl",
        primary_material_label: "Ceramic",
        secondary_material_label: None,
        source_split: "vertical-known",
        recordings: &[
            recording!(
                "vertical-known-024",
                "1",
                "622b2aea46bf100176593403",
                324625,
                "9630cded2df1519939e5e7f4f087c09a02d6e83ff908c8e05a1244c59787166d"
            ),
            recording!(
                "vertical-known-024",
                "2",
                "622b2aea46bf100179593622",
                308653,
                "c35e3cdd67e1189f36eb01ef15ea2dd32eb3629cd8039fac9a5082f78101d29f"
            ),
        ],
    },
    FrozenObject {
        source_id: "ycb-impact-vertical-object-029-plate",
        object_id: "29",
        object_name: "Plate",
        primary_material_label: "Ceramic",
        secondary_material_label: None,
        source_split: "vertical-known",
        recordings: &[
            recording!(
                "vertical-known-029",
                "1",
                "622b43f953a4e801c6517d03",
                540420,
                "53bc9549c1edb2e68fd1094a33dcbe1700fb211247e2ddda5b021ebf5b4586dc"
            ),
            recording!(
                "vertical-known-029",
                "2",
                "622b43f953a4e801c6517d09",
                307426,
                "05278dabb5367d9e482dfd8c09cd3e29861463bea7b6707d6e41da716cf0e763"
            ),
        ],
    },
    FrozenObject {
        source_id: "ycb-impact-vertical-object-025-mug",
        object_id: "25",
        object_name: "Mug",
        primary_material_label: "Ceramic",
        secondary_material_label: None,
        source_split: "vertical-unknown",
        recordings: &[
            recording!(
                "vertical-unknown-025",
                "1",
                "622b6b7e22fc5e02541f10f6",
                375632,
                "f2d1ff8d34754c48bb1a566dae63381a8f3d04b510c34e664036af9f0f364d80"
            ),
            recording!(
                "vertical-unknown-025",
                "2",
                "622b6b7d46bf100232593501",
                369391,
                "626b8ba1e03deb8802188c5f4c2a015c5c7899e65a069576d3020c945a0d54a1"
            ),
        ],
    },
    FrozenObject {
        source_id: "ycb-impact-vertical-object-026-sponge",
        object_id: "26",
        object_name: "Sponge",
        primary_material_label: "Foam",
        secondary_material_label: None,
        source_split: "vertical-known",
        recordings: &[
            recording!(
                "vertical-known-026",
                "1",
                "622b2c3222fc5e01771f10de",
                326869,
                "1e47c878cf43c97544075723028479fe14bdfa9738c41d2aa3a5c1a8fd215179"
            ),
            recording!(
                "vertical-known-026",
                "2",
                "622b2c3346bf10017f593451",
                432282,
                "8cccae4bfac98a14cd920d5db9f3ba8859c8defe8be5cf8a572665e7cca44d4c"
            ),
        ],
    },
    FrozenObject {
        source_id: "ycb-impact-vertical-object-060-washer-sponge",
        object_id: "60",
        object_name: "Washer sponge",
        primary_material_label: "Foam",
        secondary_material_label: None,
        source_split: "vertical-unknown",
        recordings: &[
            recording!(
                "vertical-unknown-060",
                "1",
                "622b6eea46bf10024459354d",
                326149,
                "305fc054d3147ec5010da56b2c9b9788c7f6cd2b43d0e124c386b1bac5b48ae5"
            ),
            recording!(
                "vertical-unknown-060",
                "2",
                "622b6eeb22fc5e025f1f1211",
                335926,
                "6b4703f5f9d065f1523c178ac08c7c5459876a85ade33910696ded9673ecd5d7"
            ),
        ],
    },
    FrozenObject {
        source_id: "ycb-impact-vertical-object-061-foam-brick",
        object_id: "61",
        object_name: "Foam brick",
        primary_material_label: "Foam",
        secondary_material_label: None,
        source_split: "vertical-unknown",
        recordings: &[
            recording!(
                "vertical-unknown-061",
                "1",
                "622b6ffd46bf1002405935ee",
                334540,
                "391bb5355f835c8c293f9ecdf6b7d309dd89d09ec849c3dc81cfbf3f6b4cf98c"
            ),
            recording!(
                "vertical-unknown-061",
                "2",
                "622b6ffd46bf100244593a63",
                375697,
                "830d8c9b153f8f17857caa108488776d0a1ce3990658c4aecfa6b87755c76e2c"
            ),
        ],
    },
    FrozenObject {
        source_id: "ycb-impact-vertical-object-027-skillet",
        object_id: "27",
        object_name: "Skillet",
        primary_material_label: "Steel",
        secondary_material_label: None,
        source_split: "vertical-known",
        recordings: &[
            recording!(
                "vertical-known-027",
                "1",
                "622b423722fc5e01b31f1c2e",
                293329,
                "5d19423c743af0b9c6537c5e664748e5267beee9967451a8e1dbb467cb438cb2"
            ),
            recording!(
                "vertical-known-027",
                "2",
                "622b423946bf1001b1594d0c",
                300185,
                "bc3efe2b6c2a9028c2e65dbb5a01d27adbee1be10fd0fe8aab9e1bbc0689e6af"
            ),
        ],
    },
    FrozenObject {
        source_id: "ycb-impact-vertical-object-048-hammer",
        object_id: "48",
        object_name: "Hammer",
        primary_material_label: "Steel",
        secondary_material_label: Some("Wood"),
        source_split: "vertical-known",
        recordings: &[
            recording!(
                "vertical-known-048",
                "1",
                "622b60d746bf10020b5932d0",
                406101,
                "20ba7bb11217fe10fe8b528e89d78bea435c7dd7f415efb3f38abca367f49fb4"
            ),
            recording!(
                "vertical-known-048",
                "2",
                "622b60d753a4e80238515a57",
                379518,
                "7864813d69b75c8d55c7feac691f83241eebfe4c4f6e02bc59226c64ac37433d"
            ),
        ],
    },
    FrozenObject {
        source_id: "ycb-impact-vertical-object-038-padlock",
        object_id: "38",
        object_name: "Padlock",
        primary_material_label: "Steel",
        secondary_material_label: None,
        source_split: "vertical-unknown",
        recordings: &[
            recording!(
                "vertical-unknown-038",
                "1",
                "622b6d5246bf10022e5934fe",
                309596,
                "d2ffe9b74b309f282316a4f4663a07c118cb98a01d13c17ffc69e20035adbf1e"
            ),
            recording!(
                "vertical-unknown-038",
                "2",
                "622b6d5322fc5e02541f192b",
                313883,
                "b713b5105a7c72b758a76155def94383a9862949ba09789a205a059e8b5d710f"
            ),
        ],
    },
    FrozenObject {
        source_id: "ycb-impact-vertical-object-020-pitcher-lid",
        object_id: "20",
        object_name: "Pitcher lid",
        primary_material_label: "Hard Plastic",
        secondary_material_label: None,
        source_split: "vertical-known",
        recordings: &[
            recording!(
                "vertical-known-020",
                "1",
                "622b28be46bf10016c593574",
                460408,
                "35c1d5590cdf33a3011aad149f84c08d7fc77d49644f98bd75ed8ee41a02a889"
            ),
            recording!(
                "vertical-known-020",
                "2",
                "622b28bd53a4e80179515d79",
                395525,
                "1b3f03c0316f6b9702160a5ed9d50c842a906db07721f034dea30df984a2565e"
            ),
        ],
    },
    FrozenObject {
        source_id: "ycb-impact-vertical-object-068-clear-box",
        object_id: "68",
        object_name: "Clear box",
        primary_material_label: "Hard Plastic",
        secondary_material_label: None,
        source_split: "vertical-known",
        recordings: &[
            recording!(
                "vertical-known-068",
                "1",
                "622b64eb46bf100217593de3",
                433675,
                "ddf9741577784be3562fd560006e022b804932a38b167ac3aa1f107ac1bf16c8"
            ),
            recording!(
                "vertical-known-068",
                "2",
                "622b64ed22fc5e023a1f1782",
                358724,
                "32107c7a4ee266dc8d1a7bef9c6c49bc4744d4c0c9b2c4b87504f526dc60d659"
            ),
        ],
    },
    FrozenObject {
        source_id: "ycb-impact-vertical-object-019-pitcher-base",
        object_id: "19",
        object_name: "Pitcher base",
        primary_material_label: "Hard Plastic",
        secondary_material_label: None,
        source_split: "vertical-unknown",
        recordings: &[
            recording!(
                "vertical-unknown-019",
                "1",
                "622b6a0422fc5e024c1f1db4",
                459904,
                "4a3beca243e79cdd3bba423acf81456661c2fc76e8d47d57c57c1141af4399dd"
            ),
            recording!(
                "vertical-unknown-019",
                "2",
                "622b6a0546bf10022359391c",
                377820,
                "8a03bb146a3e9827ccce760dde1352ab02d1734b9b561ecf1454d1b5512888ec"
            ),
        ],
    },
    FrozenObject {
        source_id: "ycb-impact-vertical-object-006-mustard-bottle",
        object_id: "6",
        object_name: "Mustard bottle",
        primary_material_label: "Soft Plastic",
        secondary_material_label: None,
        source_split: "vertical-known",
        recordings: &[
            recording!(
                "vertical-known-006",
                "1",
                "622b65e646bf100213593969",
                374333,
                "883b3f29662423d432351059223a672998bbfad49307bb783e01cd71b9e120d3"
            ),
            recording!(
                "vertical-known-006",
                "2",
                "622b65e646bf1002175942f3",
                444967,
                "5ade5ff715ac6d622233d9a74e0ee9c7bfd0f25ba022bde5713cd24da7a0fdc0"
            ),
        ],
    },
    FrozenObject {
        source_id: "ycb-impact-vertical-object-013-apple",
        object_id: "13",
        object_name: "Apple",
        primary_material_label: "Soft Plastic",
        secondary_material_label: None,
        source_split: "vertical-known",
        recordings: &[
            recording!(
                "vertical-known-013",
                "1",
                "622b26d853a4e8016a5160cd",
                300796,
                "e3eded8ddab4e1234ccb74b32cffe75d6e789586f5f2a0a7ab1be8feda61a680"
            ),
            recording!(
                "vertical-known-013",
                "2",
                "622b26d646bf10015f593572",
                331139,
                "a0d69e70b06bb13985a72d77f6d19a7070b39e7e48f367734161c74ab9999f5d"
            ),
        ],
    },
    FrozenObject {
        source_id: "ycb-impact-vertical-object-011-banana",
        object_id: "11",
        object_name: "Banana",
        primary_material_label: "Soft Plastic",
        secondary_material_label: None,
        source_split: "vertical-unknown",
        recordings: &[
            recording!(
                "vertical-unknown-011",
                "1",
                "622b68a122fc5e02491f1464",
                315751,
                "fb4865f4196f815f40acc8830a559fa2f1bf6272c3e17eb565bb3140b76f3778"
            ),
            recording!(
                "vertical-unknown-011",
                "2",
                "622b68a153a4e80244515f63",
                241455,
                "94b74236a1e9fea4fda7fb5c128409506f3b51c8a1d22af4bbdf9063c341acda"
            ),
        ],
    },
    FrozenObject {
        source_id: "ycb-impact-vertical-object-051-large-clamp",
        object_id: "51",
        object_name: "Large clamp",
        primary_material_label: "Other Plastic",
        secondary_material_label: None,
        source_split: "vertical-known",
        recordings: &[
            recording!(
                "vertical-known-051",
                "1",
                "622b611046bf10020a59340a",
                515549,
                "a411b2e4eb12f5375d9e28ffa0ee71b615ce3c60c10d19c95f58c113b7988d33"
            ),
            recording!(
                "vertical-known-051",
                "2",
                "622b610f53a4e80238515b84",
                317191,
                "293e59f71b52cc1a52927da0e3952299cac47c1514d670223284ba9036851b45"
            ),
        ],
    },
    FrozenObject {
        source_id: "ycb-impact-vertical-object-065-cups",
        object_id: "65",
        object_name: "Cups",
        primary_material_label: "Other Plastic",
        secondary_material_label: None,
        source_split: "vertical-known",
        recordings: &[
            recording!(
                "vertical-known-065",
                "1",
                "622b64a122fc5e023a1f1650",
                350430,
                "a00e13020995f4a7b1181b790536491856da4166987ce2224edde13af47aefcc"
            ),
            recording!(
                "vertical-known-065",
                "2",
                "622b64a246bf1002145939fc",
                343159,
                "ae74ddd4b9ca5350f4e86f60801d150595301a253eea40f3b515e3404d9330fe"
            ),
        ],
    },
    FrozenObject {
        source_id: "ycb-impact-vertical-object-041-small-marker",
        object_id: "41",
        object_name: "Small marker",
        primary_material_label: "Other Plastic",
        secondary_material_label: None,
        source_split: "vertical-unknown",
        recordings: &[
            recording!(
                "vertical-unknown-041",
                "1",
                "622b6d8453a4e80254516623",
                462310,
                "eaea4177b7b694d1392817e281ee8ac5346c7492bf6054bf954316a763835992"
            ),
            recording!(
                "vertical-unknown-041",
                "2",
                "622b6d8422fc5e02531f17ae",
                420131,
                "641069a568eed9290bbd447af32896ca328468ebd3a7b6e90694a54f3572f525"
            ),
        ],
    },
    FrozenObject {
        source_id: "ycb-impact-vertical-object-070-colored-wood-block",
        object_id: "70",
        object_name: "Colored wood block",
        primary_material_label: "Wood",
        secondary_material_label: None,
        source_split: "vertical-known",
        recordings: &[
            recording!(
                "vertical-known-070",
                "1",
                "622b653453a4e8023b5167a9",
                363684,
                "c555b8ba90fee920783b3410491faea7f650e6149a63dd69ca9eb948f295ffa0"
            ),
            recording!(
                "vertical-known-070",
                "2",
                "622b653322fc5e023a1f187d",
                343248,
                "ce226753a38c0ed562f7b1d52f016abaa97f7f7e29eae5e84f5b50f0203d6362"
            ),
        ],
    },
    FrozenObject {
        source_id: "ycb-impact-vertical-object-071-nine-hole-peg-test",
        object_id: "71",
        object_name: "Nine-hole peg test",
        primary_material_label: "Wood",
        secondary_material_label: None,
        source_split: "vertical-known",
        recordings: &[
            recording!(
                "vertical-known-071",
                "1",
                "622b656522fc5e02371f152d",
                296999,
                "8a9adf7cb5e8127871aaa0414e43f5893f408e5a2bc4298e213a6ee6ac9afe23"
            ),
            recording!(
                "vertical-known-071",
                "2",
                "622b656622fc5e023a1f1935",
                332878,
                "0abfb5bf6e4d0dbbf9a86d7a4ccd42d37ab92d36ab5d55af577bed071b58a2e4"
            ),
        ],
    },
    FrozenObject {
        source_id: "ycb-impact-vertical-object-036-wood-block",
        object_id: "36",
        object_name: "Wood block",
        primary_material_label: "Wood",
        secondary_material_label: None,
        source_split: "vertical-unknown",
        recordings: &[
            recording!(
                "vertical-unknown-036",
                "1",
                "622b6b9c46bf10021d593493",
                319686,
                "a1e953c661ff05492aefbcb35f942c98304172a09afe80339bf404740d26d9c1"
            ),
            recording!(
                "vertical-unknown-036",
                "2",
                "622b6b9b46bf10022d5933ae",
                341964,
                "3b2311c9a5028b27c05e747c37a205abd20eb8823059b338b5738ba4ef049bfd"
            ),
        ],
    },
];
