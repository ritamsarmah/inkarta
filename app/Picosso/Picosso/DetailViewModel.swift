//
//  DetailViewModel.swift
//  Picosso
//
//  Created by Ritam Sarmah on 3/23/23.
//

import Foundation
import PhotosUI
import SwiftUI

class DetailViewModel: ObservableObject {
    
    let title: String
    let artist: String
    let imageURL: URL
    
    init(artwork: Artwork) {
        
        self.title = artwork.title
        self.artist = artwork.artist
        
        var components = URLComponents(url: Endpoint.image.url, resolvingAgainstBaseURL: true)!
        components.queryItems = [
            .init(name: "id", value: artwork.id)
        ]
        
        imageURL = components.url!
    }
}
