//
//  GalleryViewModel.swift
//  Picosso
//
//  Created by Ritam Sarmah on 3/21/23.
//

import PhotosUI
import SwiftUI

class GalleryViewModel: ObservableObject {
    
    @Published var artworks: [Artwork]?
    @Published var next: String?
    
    @Published var isLoading = false
    @Published var isShowingUploadSheet = false
    @Published var isShowingErrorAlert = false
    
    @Published var errorMessage: String? {
        didSet {
            if oldValue == nil {
                isShowingErrorAlert = true
            }
        }
    }
    
    @Published var photosPickerItem: PhotosPickerItem? {
        didSet {
            if photosPickerItem != nil {
                isShowingUploadSheet = true
            }
        }
    }
    
    func fetch() {
        defer { self.isLoading = false }
        
        let request = URLRequest(url: Endpoint.all.url)
        Task {
            do {
                let (data, response) = try await URLSession.shared.data(for: request)
                
                try await MainActor.run {
                    if let response = response as? HTTPURLResponse, response.statusCode != 200 {
                        self.errorMessage = "Failed to fetch artwork with status: \(response.statusCode)"
                        return
                    }
                    
                    let db = try JSONDecoder().decode(FetchResponse.self, from: data)
                    self.artworks = Array(db.artworks.values).sorted(by: { $0.title < $1.title })
                    self.next = db.next
                }
            } catch let error {
                await MainActor.run {
                    self.errorMessage = error.localizedDescription
                }
            }
        }
        
    }
}
