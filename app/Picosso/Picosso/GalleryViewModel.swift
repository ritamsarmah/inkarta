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
    @Published var next: String? // TODO: Not used yet
    
    @Published var isLoading = false
    @Published var isShowingFileImporter = false
    @Published var isShowingUploadSheet = false
    
    @Published var errorInfo = ErrorAlert.Info()
    
    var uploadImageURL: URL? {
        didSet {
            if uploadImageURL != nil {
                isShowingUploadSheet = true
            }
        }
    }
    
    func fetch() async {
        let request = URLRequest(url: Endpoint.all.url)
        
        do {
            let (data, response) = try await URLSession.shared.data(for: request)
            
            try await MainActor.run {
                if let message = Utils.errorMessage(for: response, with: data) {
                    self.errorInfo.message = message
                    return
                }
                
                let db = try JSONDecoder().decode(FetchResponse.self, from: data)
                self.artworks = Array(db.artworks.values).sorted(by: { $0.title < $1.title })
                self.next = db.next
            }
        } catch let error {
            await MainActor.run {
                self.errorInfo.message = error.localizedDescription
            }
        }
    }
    
    func delete(at indexes: IndexSet) async {
        guard let artworks = artworks else { return }
        
        for index in indexes {
            let artwork = artworks[index]
            
            var components = URLComponents(url: Endpoint.delete.url, resolvingAgainstBaseURL: true)!
            components.queryItems = [
                .init(name: "id", value: artwork.id),
            ]
            
            var request = URLRequest(url: components.url!)
            request.httpMethod = "DELETE"
            
            do {
                let (data, response) = try await URLSession.shared.data(for: request)
                
                await MainActor.run {
                    if let errorMessage = Utils.errorMessage(for: response, with: data) {
                        self.errorInfo.message = errorMessage
                        return
                    }
                }
            } catch let error {
                await MainActor.run {
                    self.errorInfo.message = error.localizedDescription
                }
            }
        }
        
        await MainActor.run {
            self.artworks!.remove(atOffsets: indexes)
        }
    }
    
    func onImportFile(result: Result<URL, Error>) {
        guard let imageURL = try? result.get() else {
            errorInfo.message = "Failed to import the selected image"
            return
        }
        
        uploadImageURL = imageURL
    }
}
