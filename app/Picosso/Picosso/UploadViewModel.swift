//
//  UploadViewModel.swift
//  Picosso
//
//  Created by Ritam Sarmah on 3/23/23.
//

import Foundation
import PhotosUI
import SwiftUI

class UploadViewModel: ObservableObject {
    
    @Published private(set) var imageState: ImageState
    @Published var imageSelection: PhotosPickerItem
    
    @Published var title = "XX"
    @Published var artist = "XX"
    @Published var shouldPad = true
    @Published var shouldOverwrite = true
    
    @Published var isShowingErrorAlert = false
    
    @Published var errorMessage: String? {
        didSet {
            if oldValue == nil {
                isShowingErrorAlert = true
            }
        }
    }
    
    private var imageData: Data?
    
    init(imageSelection: PhotosPickerItem) {
        self.imageSelection = imageSelection
        self.imageState = .empty
        
        let progress = loadTransferable(from: imageSelection)
        self.imageState = .loading(progress)
    }
    
    func upload() async -> Bool {
        guard let imageData else { return false }
        
        var components = URLComponents(url: Endpoint.upload.url, resolvingAgainstBaseURL: true)!
        components.queryItems = [
            .init(name: "title", value: title),
            .init(name: "artist", value: artist),
            .init(name: "pad", value: "\(shouldPad)")
            .init(name: "", value: "\(shouldPad)")
        ]
        
        var request = URLRequest(url: Endpoint.upload.url)
        request.httpMethod = "POST"
        
        do {
            // TODO upload as file instead of data?
            let (data, response) = try await URLSession.shared.upload(for: request, from: imageData)
            
            return await MainActor.run {
                if let response = response as? HTTPURLResponse, response.statusCode != 200,
                   let message = String(data: data, encoding: .utf8) {
                    self.errorMessage = message
                    return false
                }
                
                return true
            }
        } catch let error {
            return await MainActor.run {
                self.errorMessage = error.localizedDescription
                return false
            }
        }
    }
    
    private func loadTransferable(from imageSelection: PhotosPickerItem) -> Progress {
        return imageSelection.loadTransferable(type: ProfileImage.self) { result in
            DispatchQueue.main.async {
                switch result {
                case .success(let profileImage?):
                    self.imageState = .success(profileImage.image)
                    self.imageData = profileImage.data
                case .success(nil):
                    self.imageState = .empty
                case .failure(let error):
                    self.imageState = .failure(error)
                }
            }
        }
    }
}

extension UploadViewModel {
    enum ImageState {
        case empty
        case loading(Progress)
        case success(Image)
        case failure(Error)
    }

    enum TransferError: Error {
        case importFailed
    }
}

extension UploadViewModel {
    struct ProfileImage: Transferable {
        let image: Image
        let data: Data
        
        static var transferRepresentation: some TransferRepresentation {
            DataRepresentation(importedContentType: .image) { data in
                guard let uiImage = UIImage(data: data) else {
                    throw TransferError.importFailed
                }
                let image = Image(uiImage: uiImage)
                return ProfileImage(image: image, data: data)
            }
        }
    }
}
