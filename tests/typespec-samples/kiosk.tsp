namespace KioskExample;

interface Display {
  createKiosk(...Kiosk): Kiosk;
  listKiosks(): Kiosk[];
  getKiosk(id: int32): Kiosk;
  deleteKiosk(id: int32): void;
}

model Kiosk {
  id: int32;
  name: string;
  location: string;
  isActive: boolean;
}

model DisplayMessage {
  id: int32;
  content: string;
  kioskId: int32;
}
