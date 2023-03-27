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
    
    let artwork: Artwork
    let imageURL: URL
    
    init(artwork: Artwork) {
        self.artwork = artwork
        
        var components = URLComponents(url: Endpoint.image.url, resolvingAgainstBaseURL: true)!
        components.queryItems = [
            .init(name: "id", value: artwork.id)
        ]
        
        self.imageURL = components.url!
    }
}
