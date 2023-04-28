//
//  Endpoint.swift
//  Inkasso
//
//  Created by Ritam Sarmah on 3/23/23.
//

import Foundation

enum Endpoint: String {
    private static let baseURL = URL(string: "http://192.168.1.5:5000")!
    
    case all = "/all"       // Get all data
    case image = "/image"   // Retrieve image for artwork
    case upload = "/upload" // Upload artwork
    case delete = "/delete" // Delete artwork
    case next = "/next"     // Retrieve or update next random image for /image
    
    var url: URL {
        return Self.baseURL.appendingPathComponent(self.rawValue)
    }
}

struct FetchResponse: Codable {
    let artworks: [String: Artwork]
    let next: String
}
