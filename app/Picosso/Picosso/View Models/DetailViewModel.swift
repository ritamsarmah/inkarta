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
    
    let parentViewModel: GalleryViewModel // Reference to parent view model to update data after changes
    
    @Published var isNext: Bool
    @Published var errorInfo = ErrorAlert.Info()
    
    init(artwork: Artwork, parentViewModel: GalleryViewModel) {
        self.artwork = artwork
        self.parentViewModel = parentViewModel
        self.isNext = artwork.id == parentViewModel.nextId
        
        var components = URLComponents(url: Endpoint.image.url, resolvingAgainstBaseURL: true)!
        components.queryItems = [
            .init(name: "id", value: artwork.id)
        ]
        
        self.imageURL = components.url!
    }
    
    func setNextId() async {
        var components = URLComponents(url: Endpoint.next.url, resolvingAgainstBaseURL: true)!
        components.queryItems = [
            .init(name: "id", value: artwork.id),
        ]
        
        var request = URLRequest(url: components.url!)
        request.httpMethod = "PUT"
        
        do {
            let (data, response) = try await URLSession.shared.data(for: request)
            
            await MainActor.run {
                if let message = Utils.errorMessage(for: response, with: data) {
                    self.errorInfo.message = message
                    return
                }
                
                self.parentViewModel.nextId = self.artwork.id
                self.isNext = true
            }
        } catch let error {
            await MainActor.run {
                self.errorInfo.message = error.localizedDescription
            }
        }
    }
}
