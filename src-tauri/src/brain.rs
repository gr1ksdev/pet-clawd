// Pet Clawd — núcleo do pet (port do ClawdBrain do plugin clawd_pet_pt).
// Estado vive em JSON no app data dir; decay é CALCULADO a partir de
// timestamps (sem serviço em background = economia de bateria).
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Stats {
    pub pets: u64,
    pub eaten: u64,
    pub cleaned: u64,
    pub toys: u64,
    pub jumps: u64,
    pub sessions: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Inventory {
    pub fish: u32,
    pub burger: u32,
    pub coffee: u32,
    pub cookie: u32,
    pub ball: u32,
    pub yarn: u32,
    pub mouse: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PetState {
    pub mood: i32,      // 0..100 humor
    pub bond: i32,      // 0..100 vínculo
    pub hunger: i32,    // 0..100 (0 = de barriga cheia)
    pub energy: i32,    // 0..100
    pub mess: u32,      // sujeiras acumuladas
    pub first_met: u64, // epoch s
    pub last_tick: u64, // epoch s do último decay
    pub last_poop: u64,
    pub last_restock: u64,
    pub stats: Stats,
    pub inventory: Inventory,
}

impl Default for PetState {
    fn default() -> Self {
        let now = Self::now();
        Self {
            mood: 75,
            bond: 30,
            hunger: 20,
            energy: 90,
            mess: 0,
            first_met: now,
            last_tick: now,
            last_poop: 0,
            last_restock: 0,
            stats: Stats::default(),
            inventory: Inventory {
                fish: 5,
                burger: 5,
                coffee: 5,
                cookie: 5,
                ball: 3,
                yarn: 3,
                mouse: 3,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ActResult {
    pub state: PetState,
    pub speech: String,
}

impl PetState {
    pub fn now() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }

    fn clamp(&mut self) {
        self.mood = self.mood.clamp(0, 100);
        self.bond = self.bond.clamp(0, 100);
        self.hunger = self.hunger.clamp(0, 100);
        self.energy = self.energy.clamp(0, 100);
    }

    /// Decay passivo por horas reais desde o último tick (cap 7 dias).
    pub fn tick(&mut self) {
        let now = Self::now();
        let dt = now.saturating_sub(self.last_tick).min(60 * 60 * 24 * 7) as i64;
        if dt <= 0 {
            self.last_tick = now;
            return;
        }
        let hours = dt as f64 / 3600.0;
        self.mood -= (2.0 * hours).round() as i32;
        self.hunger += (3.0 * hours).round() as i32;
        self.energy -= (1.5 * hours).round() as i32;
        if self.hunger >= 90 {
            self.mood -= (4.0 * hours).round() as i32;
        }
        if self.mess >= 3 {
            self.mood -= (2.0 * hours).round() as i32;
        }
        // sujeirinha quando a barriga fica cheia (máx 1 a cada 6h, limite 5)
        if self.hunger >= 80
            && now.saturating_sub(self.last_poop) > 6 * 3600
            && self.mess < 5
        {
            self.mess += 1;
            self.mood -= 4;
            self.last_poop = now;
        }
        // reposição diária da mochila
        if self.last_restock == 0 || now.saturating_sub(self.last_restock) > 24 * 3600 {
            self.inventory = Inventory {
                fish: 5,
                burger: 5,
                coffee: 5,
                cookie: 5,
                ball: 3,
                yarn: 3,
                mouse: 3,
            };
            self.last_restock = now;
        }
        self.clamp();
        self.last_tick = now;
    }

    pub fn days_together(&self) -> u64 {
        Self::now().saturating_sub(self.first_met) / 86_400 + 1
    }

    /// Rótulos de humor (mesma vibe dos badges do plugin).
    pub fn mood_label(&self) -> &'static str {
        let (m, h, e) = (self.mood, self.hunger, self.energy);
        if e < 20 {
            "😴 Sonolento"
        } else if h >= 85 {
            "😿 Faminto"
        } else if m < 25 {
            "😢 Tristinho"
        } else if m < 50 {
            "🙂 Tranquilo"
        } else if m < 80 {
            "😊 Contente"
        } else {
            "🥰 Radiante"
        }
    }

    /// Fala padrão (local), escolhida pelo estado — a IA substitui quando
    /// configurada.
    pub fn idle_speech(&self) -> String {
        let pool: &[&str] = if self.mess > 0 {
            &[
                "Ops… fiz sujeirinha de novo 😿",
                "Era sem querer, eu juro!",
            ]
        } else if self.hunger >= 80 {
            &[
                "Tô com a barriga roncando… 🍔",
                "Um petisco cairia bem agora!",
            ]
        } else if self.energy < 25 {
            &["Dormindo com um olho aberto… 😴", "Soneca rápida?"]
        } else if self.bond >= 85 {
            &[
                "Você é meu humano favorito! ❤️",
                "Melhores amigos para sempre! 💕",
                "Miau! Sinto sua falta quando sai!",
            ]
        } else if self.mood >= 60 {
            &[
                "Olá! Que bom te ver por aqui! ♥",
                "Estou adorando te acompanhar!",
                "Pronto para aprender coisas novas!",
            ]
        } else {
            &["Preciso de um carinho… 🥺", "Saudades de brincar!"]
        };
        let idx = (Self::now() as usize) % pool.len();
        pool[idx].to_string()
    }

    /// Ações do pet — mesmas do plugin (actPet/actFeed/actToy/actClean/…).
    pub fn act(&mut self, kind: &str, item: Option<&str>) -> ActResult {
        self.tick();
        let mut speech = String::new();
        match kind {
            "pet" => {
                self.mood += 6;
                self.bond += 1;
                self.stats.pets += 1;
                speech = "Clawd ronronou com seu carinho! ❤️".into();
            }
            "feed" => {
                let it = item.unwrap_or("treat");
                let have = match it {
                    "fish" => self.inventory.fish,
                    "burger" => self.inventory.burger,
                    "coffee" => self.inventory.coffee,
                    "cookie" => self.inventory.cookie,
                    _ => u32::MAX,
                };
                if it != "treat" && have == 0 {
                    speech = "Acabaram os petiscos — volte amanhã! 🍽️".into();
                } else {
                    match it {
                        "fish" => {
                            self.inventory.fish -= 1;
                            self.hunger -= 30;
                        }
                        "burger" => {
                            self.inventory.burger -= 1;
                            self.hunger -= 40;
                        }
                        "coffee" => {
                            self.inventory.coffee -= 1;
                            self.hunger -= 10;
                            self.energy += 25;
                        }
                        "cookie" => {
                            self.inventory.cookie -= 1;
                            self.hunger -= 15;
                            self.mood += 3;
                        }
                        _ => {
                            self.hunger -= 20;
                        }
                    }
                    self.mood += 8;
                    self.bond += 1;
                    self.stats.eaten += 1;
                    // comeu demais → sujeirinha
                    if self.hunger <= 15 && self.mess < 5 {
                        self.mess += 1;
                    }
                    speech = "Miau! Esse petisco estava delicioso! 🐟".into();
                }
            }
            "toy" => {
                let it = item.unwrap_or("ball");
                let have = match it {
                    "ball" => self.inventory.ball,
                    "yarn" => self.inventory.yarn,
                    _ => self.inventory.mouse,
                };
                if have == 0 {
                    speech = "Esse brinquedo tá em falta hoje! 🧸".into();
                } else {
                    match it {
                        "ball" => self.inventory.ball -= 1,
                        "yarn" => self.inventory.yarn -= 1,
                        _ => self.inventory.mouse -= 1,
                    }
                    self.mood += 10;
                    self.bond += 1;
                    self.energy -= 5;
                    self.stats.toys += 1;
                    speech = "Brinquedo arremessado! Clawd correu atrás! 🎾".into();
                }
            }
            "jump" => {
                self.mood += 4;
                self.energy -= 3;
                self.stats.jumps += 1;
                speech = "Boing! Pulo incrível! 🐾".into();
            }
            "clean" => {
                if self.mess > 0 {
                    self.mess -= 1;
                    self.stats.cleaned += 1;
                    self.mood += 3;
                    speech = "Tudo limpinho e brilhando! ✨".into();
                } else {
                    speech = "Já tá tudo limpo! 🧼".into();
                }
            }
            "sleep" => {
                self.energy += 30;
                self.mood += 2;
                speech = "Clawd se aninhou para uma soneca… 💤".into();
            }
            _ => {
                speech = "Miau? 🐾".into();
            }
        }
        self.clamp();
        ActResult {
            state: self.clone(),
            speech,
        }
    }
}
