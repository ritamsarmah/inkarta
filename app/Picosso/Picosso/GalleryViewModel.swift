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
    @Published var isShowingFileImporter = false
    @Published var isShowingUploadSheet = false
    @Published var isShowingErrorAlert = false
    
    @Published var errorMessage: String? {
        didSet {
            if oldValue == nil {
                isShowingErrorAlert = true
            }
        }
    }
    
    var newImageURL: URL? {
        didSet {
            if newImageURL != nil {
                isShowingUploadSheet = true
            }
        }
    }
    
    func fetch() {
        isLoading = true
        
        let request = URLRequest(url: Endpoint.all.url)
        Task {
            do {
                let (data, response) = try await URLSession.shared.data(for: request)

                try await MainActor.run {
                    if let response = response as? HTTPURLResponse, response.statusCode != 200 {
                        self.errorMessage = "Failed to fetch artwork with status: \(response.statusCode)"
                        self.isLoading = false
                        return
                    }

                    let db = try JSONDecoder().decode(FetchResponse.self, from: data)
                    self.artworks = Array(db.artworks.values).sorted(by: { $0.title < $1.title })
                    self.next = db.next
                    self.isLoading = false
                }
            } catch let error {
                await MainActor.run {
                    self.errorMessage = error.localizedDescription
                    self.isLoading = false
                }
            }
        }
    }
}
