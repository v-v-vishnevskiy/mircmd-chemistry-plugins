// Copyright (c) 2026 Valery Vishnevskiy and Yury Vishnevskiy
// Licensed under the Apache 2.0 License

export interface AtomicCoordinates {
  atomic_num: number[];
  x: number[];
  y: number[];
  z: number[];
}

export interface Element {
  atomic_number: number;
  symbol: string;
  covalent_radius: number;
}
