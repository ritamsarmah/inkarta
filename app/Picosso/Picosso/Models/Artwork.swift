//
//  Artwork.swift
//  Picosso
//
//  Created by Ritam Sarmah on 3/23/23.
//

struct Artwork: Codable, Identifiable {
    let id: String
    let title: String
    let artist: String
    let dark: Bool
}
